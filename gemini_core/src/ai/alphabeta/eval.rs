use crate::engine::position::Position;
use crate::engine::types::Color;

use super::{CORNER_MASK, DISC_SCALE};

/// 非終局の評価関数（手番視点）。
pub(super) fn evaluate(position: Position) -> i32 {
    let empty = i32::from(empty_count(position));
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

    let (w_corner, w_mobility, w_material) = if empty > 44_i32 {
        (30_i32, 5_i32, 0_i32)
    } else if empty > 20_i32 {
        (30_i32, 3_i32, 1_i32)
    } else {
        (20_i32, 1_i32, 5_i32)
    };

    let mut score: i32 = 0;
    score = score.wrapping_add(corners.wrapping_mul(w_corner));
    score = score.wrapping_add(mobility.wrapping_mul(w_mobility));
    score = score.wrapping_add(material.wrapping_mul(w_material));
    score
}

/// 終局時（双方パス）の評価（手番視点）。
pub(super) fn terminal_score(position: Position) -> i32 {
    let side = position.side_to_move();
    let (black, white) = position.counts();
    let (player, opponent) = match side {
        Color::Black => (black, white),
        Color::White => (white, black),
    };
    let diff = diff_i32(player, opponent);
    diff.wrapping_mul(DISC_SCALE)
}

/// 空きマス数。
pub(super) fn empty_count(position: Position) -> u8 {
    let occupied = position.occupied();
    let empty_u32 = 64_u32.wrapping_sub(occupied.count_ones());
    u8::try_from(empty_u32).unwrap_or(u8::MAX)
}

/// `u32` 同士の差を `i32` として返す。
fn diff_i32(lhs: u32, rhs: u32) -> i32 {
    let ai = i32::try_from(lhs).unwrap_or(i32::MAX);
    let bi = i32::try_from(rhs).unwrap_or(i32::MAX);
    ai.wrapping_sub(bi)
}
