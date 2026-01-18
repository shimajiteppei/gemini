use super::INF;
use super::eval::terminal_score;
use super::limits::{SearchContext, SearchLimits};
use super::search::{negamax, search_root};
use super::tt::{TranspositionTable, Zobrist};
use crate::ai::random;
use crate::ai::types::Ai as _;
use crate::ai::types::Move;
use crate::engine::position::Position;
use crate::engine::types::Color;

const TEST_TT_SIZE: usize = 1 << 10;

#[test]
fn terminal_score_sign_is_from_side_to_move_perspective() {
    // 終局（盤面が埋まっている）かつ黒の勝ち。
    let full_black = u64::MAX;
    let empty_white = u64::MIN;

    let pos_black_to_move = Position::from_raw(full_black, empty_white, Color::Black);
    let pos_white_to_move = Position::from_raw(full_black, empty_white, Color::White);

    assert_eq!(pos_black_to_move.legal_moves(), u64::MIN);
    assert_eq!(pos_white_to_move.legal_moves(), u64::MIN);

    assert!(terminal_score(pos_black_to_move) > 0_i32);
    assert!(terminal_score(pos_white_to_move) < 0_i32);
    assert_eq!(
        terminal_score(pos_black_to_move),
        -terminal_score(pos_white_to_move)
    );
}

fn find_position_where_current_player_must_pass() -> Option<Position> {
    // 決定的に見つかるまで seed を変えつつ探索する。
    for seed in 0_u64..256 {
        let mut agent = random::Agent::new(seed);
        let mut pos = Position::initial();
        let mut illegal_move_chosen = false;

        // 最大 60 手程度で終局するので余裕を持たせる。
        for _ply in 0_u16..100 {
            let side = pos.side_to_move();
            let my_moves = pos.legal_moves_for(side);
            let opp_moves = pos.legal_moves_for(side.opponent());

            // 相手は打てるが自分は打てない（＝パスが必要）。
            if my_moves == u64::MIN && opp_moves != u64::MIN {
                return Some(pos);
            }

            // 終局（双方パス）に到達したらこの seed は諦める。
            if my_moves == u64::MIN && opp_moves == u64::MIN {
                break;
            }

            let mv = agent.select_move(pos);
            let next = match mv {
                Move::Pass => Ok(pos.pass()),
                Move::Place(square) => pos.apply_move(square),
            };

            pos = match next {
                Ok(value) => value,
                Err(_err) => {
                    illegal_move_chosen = true;
                    break;
                }
            };
        }

        assert!(!illegal_move_chosen, "random agent chose illegal move");
    }

    None
}

#[test]
fn negamax_performs_pass_when_no_legal_moves() {
    let pos_opt = find_position_where_current_player_must_pass();
    assert!(
        pos_opt.is_some(),
        "pass position not found in deterministic search"
    );
    let pos = pos_opt.unwrap_or_else(Position::initial);

    let side = pos.side_to_move();
    assert_eq!(pos.legal_moves_for(side), u64::MIN);
    assert_ne!(pos.legal_moves_for(side.opponent()), u64::MIN);

    let depth = 4;
    let limits = SearchLimits::new(depth, u64::MAX);
    let mut tt = TranspositionTable::new(TEST_TT_SIZE);
    let zobrist = Zobrist::new();
    let mut ctx = SearchContext::new(limits, &mut tt, &zobrist);

    let mut aborted = false;
    let score = negamax(pos, depth, -INF, INF, &mut ctx).unwrap_or_else(|_| {
        aborted = true;
        0_i32
    });
    assert!(!aborted, "search aborted unexpectedly");

    aborted = false;
    let v_expected_inner =
        negamax(pos.pass(), depth - 1, -INF, INF, &mut ctx).unwrap_or_else(|_| {
            aborted = true;
            0_i32
        });
    assert!(!aborted, "search aborted unexpectedly");
    let v_expected = -v_expected_inner;
    assert_eq!(score, v_expected);
}

#[test]
fn tt_hits_increase_when_searching_same_position_twice() {
    let position = Position::initial();

    let mut tt = TranspositionTable::new(TEST_TT_SIZE);
    let zobrist = Zobrist::new();
    let limits = SearchLimits::new(4, 1_000_000);

    let r1 = search_root(position, limits, &mut tt, &zobrist);
    let r2 = search_root(position, limits, &mut tt, &zobrist);

    assert!(matches!(r1.best_move(), Move::Place(_)));
    assert!(matches!(r2.best_move(), Move::Place(_)));
    assert!(r1.completed_depth() >= 1);
    assert!(r2.completed_depth() >= 1);
    assert!(r1.best_score() > -INF);
    assert!(r2.best_score() > -INF);
    assert!(r2.stats().tt_hits() >= r1.stats().tt_hits());
}
