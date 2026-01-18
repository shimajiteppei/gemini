use crate::engine::position::Position;
use crate::engine::types::{Color, Square};

use super::CORNER_MASK;

/// 1ビットのビットボードから `Square` を生成する。
pub(super) fn square_from_bit(bit: u64) -> Option<Square> {
    if bit == u64::MIN {
        return None;
    }

    let index_u32 = bit.trailing_zeros();
    let index_u8 = match u8::try_from(index_u32) {
        Ok(value) => value,
        Err(_) => return None,
    };

    Some(Square::from_index_unchecked(index_u8))
}

/// X-square から対応するコーナー（A1/H1/A8/H8）を返す。
const fn corner_for_x_square(index: u8) -> Option<u8> {
    match index {
        9 => Some(0),
        14 => Some(7),
        49 => Some(56),
        54 => Some(63),
        _ => None,
    }
}

/// C-square から対応するコーナー（A1/H1/A8/H8）を返す。
const fn corner_for_c_square(index: u8) -> Option<u8> {
    match index {
        1 | 8 => Some(0),
        6 | 15 => Some(7),
        48 | 57 => Some(56),
        55 | 62 => Some(63),
        _ => None,
    }
}

/// 合法手を簡易評価でソートして返す。
pub(super) fn order_moves(
    position: &Position,
    legal_moves: u64,
    tt_move: Option<Square>,
) -> Vec<Square> {
    let side = position.side_to_move();
    let player_bb = match side {
        Color::Black => position.black(),
        Color::White => position.white(),
    };

    let cap_u32 = legal_moves.count_ones();
    let cap = match usize::try_from(cap_u32) {
        Ok(value) => value,
        Err(_err) => usize::MAX,
    };
    let mut moves: Vec<(i32, Square)> = Vec::with_capacity(cap);
    let mut bb = legal_moves;

    while bb != u64::MIN {
        let choice = bb & bb.wrapping_neg();
        let mv = if let Some(value) = square_from_bit(choice) {
            value
        } else {
            bb &= bb.wrapping_sub(1);
            continue;
        };

        let mut score: i32 = 0;
        if Some(mv) == tt_move {
            score = score.wrapping_add(1_000_000);
        }

        let mv_bit = mv.bit();
        if (mv_bit & CORNER_MASK) != u64::MIN {
            score = score.wrapping_add(100_000);
        } else {
            let mv_index = mv.index();
            if let Some(corner_index) = corner_for_x_square(mv_index) {
                let corner = Square::from_index_unchecked(corner_index);
                if (player_bb & corner.bit()) == u64::MIN {
                    score = score.wrapping_sub(50_000);
                }
            } else if let Some(corner_index) = corner_for_c_square(mv_index) {
                let corner = Square::from_index_unchecked(corner_index);
                if (player_bb & corner.bit()) == u64::MIN {
                    score = score.wrapping_sub(20_000);
                }
            } else {
                // no-op
            }
        }

        // cheap heuristic: 相手番での合法手数（少ないほど良い）
        let opp_mobility = match position.apply_move(mv) {
            Ok(next) => i32::try_from(next.legal_moves().count_ones()).unwrap_or(i32::MAX),
            Err(_err) => 0_i32,
        };
        score = score.wrapping_sub(opp_mobility);

        moves.push((score, mv));
        bb &= bb.wrapping_sub(1);
    }

    moves.sort_by(|&(score_a, mv_a), &(score_b, mv_b)| {
        score_b
            .cmp(&score_a)
            .then_with(|| mv_a.index().cmp(&mv_b.index()))
    });

    moves.into_iter().map(|(_, mv)| mv).collect()
}
