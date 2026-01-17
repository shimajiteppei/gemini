use crate::ai::types::{Ai, Move};
use crate::engine::position::Position;
use crate::engine::types::{Color, Square};
use core::cmp::Ordering;

/// 角マス（4隅）のマスク。
const CORNER_MASK: u64 = 0x8100_0000_0000_0081;

/// 終局時の勝敗評価の基準点。
const SCORE_WIN: i32 = 10_000;

/// 角の重み。
const WEIGHT_CORNER: i32 = 25;

/// モビリティ（合法手数）の重み。
const WEIGHT_MOBILITY: i32 = 2;

/// 石差の重み。
const WEIGHT_MATERIAL: i32 = 1;

/// アルファベータ探索を行うAI。
#[derive(Debug)]
#[non_exhaustive]
pub struct Agent {
    /// 探索深さ。
    depth: u8,
}

impl Agent {
    /// 探索深さを返す。
    #[inline]
    #[must_use]
    pub const fn depth(self) -> u8 {
        self.depth
    }

    /// `depth` を指定して初期化する。
    #[inline]
    #[must_use]
    pub const fn new(depth: u8) -> Self {
        Self { depth }
    }
}

impl Ai for Agent {
    #[inline]
    fn select_move(&mut self, position: Position) -> Move {
        let depth = normalize_depth(self.depth);
        select_best_move(position, depth)
    }
}

/// 探索深さを正規化する（0の場合は1にする）。
#[inline]
const fn normalize_depth(depth: u8) -> u8 {
    if depth == u8::MIN {
        u8::MIN.wrapping_add(1)
    } else {
        depth
    }
}

/// 現局面から最善手を探索して返す。
fn select_best_move(position: Position, depth: u8) -> Move {
    let legal_moves = position.legal_moves();
    if legal_moves == u64::MIN {
        return Move::Pass;
    }

    let mut best_score = i32::MIN;
    let mut best_square: Option<Square> = None;
    let mut bb = legal_moves;

    let alpha_start = i32::MIN;
    let beta_start = i32::MAX;
    let next_depth = depth.wrapping_sub(1);

    while bb != u64::MIN {
        let choice = bb & bb.wrapping_neg();
        let square_opt = square_from_bit(choice);

        let square = if let Some(value) = square_opt {
            value
        } else {
            bb &= bb.wrapping_sub(1);
            continue;
        };

        let next = match position.apply_move(square) {
            Ok(value) => value,
            Err(_err) => {
                bb &= bb.wrapping_sub(1);
                continue;
            }
        };

        let score = negamax(
            next,
            next_depth,
            beta_start.wrapping_neg(),
            alpha_start.wrapping_neg(),
        )
        .wrapping_neg();
        if score > best_score {
            best_score = score;
            best_square = Some(square);
        }

        bb &= bb.wrapping_sub(1);
    }

    best_square.map_or(Move::Pass, Move::Place)
}

/// 1ビットのビットボードから `Square` を生成する。
fn square_from_bit(bit: u64) -> Option<Square> {
    if bit == u64::MIN {
        return None;
    }

    let index_u32 = bit.trailing_zeros();
    let index_u8 = match u8::try_from(index_u32) {
        Ok(value) => value,
        Err(_conversion_error) => return None,
    };

    Some(Square::from_index_unchecked(index_u8))
}

/// ネガマックス（αβ付き）。
fn negamax(position: Position, depth: u8, alpha: i32, beta: i32) -> i32 {
    if depth == u8::MIN {
        return evaluate(position);
    }

    let legal_moves = position.legal_moves();
    if legal_moves == u64::MIN {
        let opponent = position.side_to_move().opponent();
        if position.legal_moves_for(opponent) == u64::MIN {
            return evaluate_terminal(position);
        }

        let passed = position.pass();
        let next_depth = depth.wrapping_sub(1);
        return negamax(
            passed,
            next_depth,
            beta.wrapping_neg(),
            alpha.wrapping_neg(),
        )
        .wrapping_neg();
    }

    let mut best = i32::MIN;
    let mut alpha_mut = alpha;
    let mut bb = legal_moves;
    let next_depth = depth.wrapping_sub(1);

    while bb != u64::MIN {
        let choice = bb & bb.wrapping_neg();
        let square_opt = square_from_bit(choice);

        let square = if let Some(value) = square_opt {
            value
        } else {
            bb &= bb.wrapping_sub(1);
            continue;
        };

        let next = match position.apply_move(square) {
            Ok(value) => value,
            Err(_err) => {
                bb &= bb.wrapping_sub(1);
                continue;
            }
        };

        let score = negamax(
            next,
            next_depth,
            beta.wrapping_neg(),
            alpha_mut.wrapping_neg(),
        )
        .wrapping_neg();
        if score > best {
            best = score;
        }

        if best > alpha_mut {
            alpha_mut = best;
        }

        if alpha_mut >= beta {
            break;
        }

        bb &= bb.wrapping_sub(1);
    }

    best
}

/// 非終局の評価関数。
fn evaluate(position: Position) -> i32 {
    let side = position.side_to_move();
    let (player_bb, opponent_bb) = match side {
        Color::Black => (position.black(), position.white()),
        Color::White => (position.white(), position.black()),
    };

    let material = diff_i32(player_bb.count_ones(), opponent_bb.count_ones());
    let corners = diff_i32(
        (player_bb & CORNER_MASK).count_ones(),
        (opponent_bb & CORNER_MASK).count_ones(),
    );

    let mobility = diff_i32(
        position.legal_moves_for(side).count_ones(),
        position.legal_moves_for(side.opponent()).count_ones(),
    );

    let mut score: i32 = 0;
    score = score.wrapping_add(material.wrapping_mul(WEIGHT_MATERIAL));
    score = score.wrapping_add(corners.wrapping_mul(WEIGHT_CORNER));
    score = score.wrapping_add(mobility.wrapping_mul(WEIGHT_MOBILITY));
    score
}

/// 終局時（双方パス）の評価。
fn evaluate_terminal(position: Position) -> i32 {
    let side = position.side_to_move();
    let (black, white) = position.counts();
    let (player, opponent) = match side {
        Color::Black => (black, white),
        Color::White => (white, black),
    };

    let diff = diff_i32(player, opponent);
    match diff.cmp(&0) {
        Ordering::Greater => SCORE_WIN.wrapping_neg(),
        Ordering::Less => SCORE_WIN,
        Ordering::Equal => 0,
    }
}

/// `u32` 同士の差を `i32` として返す。
fn diff_i32(lhs: u32, rhs: u32) -> i32 {
    let ai = i32::try_from(lhs).unwrap_or(i32::MAX);
    let bi = i32::try_from(rhs).unwrap_or(i32::MAX);
    ai.wrapping_sub(bi)
}
