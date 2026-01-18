use crate::engine::types::{Color, Square};

/// A列（x = 0）のマスク。
const FILE_A: u64 = 0x0101_0101_0101_0101;

/// H列（x = 7）のマスク。
const FILE_H: u64 = 0x8080_8080_8080_8080;

/// 盤面の拡張（Kogge-Stone）を行う反復回数。
const SPREAD_STEPS: u8 = 5;

/// 1ビット分のシフト量。
const SHIFT_1: u32 = 1;

/// 7ビット分のシフト量。
const SHIFT_7: u32 = 7;

/// 8ビット分のシフト量。
const SHIFT_8: u32 = 8;

/// 9ビット分のシフト量。
const SHIFT_9: u32 = 9;

/// 初期配置（黒）の1つ目。
const START_BLACK_0: u32 = 28;

/// 初期配置（黒）の2つ目。
const START_BLACK_1: u32 = 35;

/// 初期配置（白）の1つ目。
const START_WHITE_0: u32 = 27;

/// 初期配置（白）の2つ目。
const START_WHITE_1: u32 = 36;

/// `u64` の 1 を表す値。
const U64_ONE: u64 = u64::MIN.wrapping_add(1);

/// 局面（盤面＋手番）。
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Position {
    /// 黒石のビットボード。
    black: u64,
    /// 手番。
    side_to_move: Color,
    /// 白石のビットボード。
    white: u64,
}

/// 着手の適用に失敗した理由。
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ApplyMoveError {
    /// 指定マスが合法手ではない。
    IllegalMove,
}

impl Position {
    /// 着手を適用する。
    ///
    /// # Errors
    ///
    /// 指定されたマスが合法手でない場合、`ApplyMoveError::IllegalMove` を返す。
    ///
    #[inline]
    pub fn apply_move(self, square: Square) -> Result<Self, ApplyMoveError> {
        let legal = self.legal_moves();
        if legal & square.bit() == u64::MIN {
            return Err(ApplyMoveError::IllegalMove);
        }

        let (player, opponent) = match self.side_to_move {
            Color::Black => (self.black, self.white),
            Color::White => (self.white, self.black),
        };

        let flipped = flips(player, opponent, square);
        let next_player = player | square.bit() | flipped;
        let next_opponent = opponent & !flipped;

        let (black, white) = match self.side_to_move {
            Color::Black => (next_player, next_opponent),
            Color::White => (next_opponent, next_player),
        };

        Ok(Self {
            black,
            side_to_move: self.side_to_move.opponent(),
            white,
        })
    }

    /// 黒石のビットボードを返す。
    #[inline]
    #[must_use]
    pub const fn black(self) -> u64 {
        self.black
    }

    /// 指定手番で着手可能かを返す。
    #[inline]
    #[must_use]
    pub fn can_play_for(self, color: Color) -> bool {
        self.legal_moves_for(color) != u64::MIN
    }

    /// 石数（黒、白）を返す。
    #[inline]
    #[must_use]
    pub const fn counts(self) -> (u32, u32) {
        (self.black.count_ones(), self.white.count_ones())
    }

    /// 盤面を生のビットボードから生成する（crate 内部向け）。
    ///
    /// - `black` と `white` は重複しないこと（`black & white == 0`）
    /// - 盤面の妥当性（合法手が存在するか等）は呼び出し側が保証する
    #[cfg(test)]
    #[inline]
    #[must_use]
    pub(crate) const fn from_raw(black: u64, white: u64, side_to_move: Color) -> Self {
        Self {
            black,
            side_to_move,
            white,
        }
    }

    /// 初期局面を返す。
    #[inline]
    #[must_use]
    pub const fn initial() -> Self {
        let b0 = match U64_ONE.checked_shl(START_BLACK_0) {
            Some(value) => value,
            None => u64::MIN,
        };
        let b1 = match U64_ONE.checked_shl(START_BLACK_1) {
            Some(value) => value,
            None => u64::MIN,
        };
        let w0 = match U64_ONE.checked_shl(START_WHITE_0) {
            Some(value) => value,
            None => u64::MIN,
        };
        let w1 = match U64_ONE.checked_shl(START_WHITE_1) {
            Some(value) => value,
            None => u64::MIN,
        };

        Self {
            black: b0 | b1,
            side_to_move: Color::Black,
            white: w0 | w1,
        }
    }

    /// 現手番の合法手ビットボードを返す。
    #[inline]
    #[must_use]
    pub fn legal_moves(self) -> u64 {
        self.legal_moves_for(self.side_to_move)
    }

    /// 指定手番の合法手ビットボードを返す。
    #[inline]
    #[must_use]
    pub fn legal_moves_for(self, color: Color) -> u64 {
        let (player, opponent) = match color {
            Color::Black => (self.black, self.white),
            Color::White => (self.white, self.black),
        };

        legal_moves(player, opponent)
    }

    /// 盤面の占有ビットボードを返す。
    #[inline]
    #[must_use]
    pub const fn occupied(self) -> u64 {
        self.black | self.white
    }

    /// パス（手番交代）を適用する。
    #[inline]
    #[must_use]
    pub const fn pass(self) -> Self {
        Self {
            black: self.black,
            side_to_move: self.side_to_move.opponent(),
            white: self.white,
        }
    }

    /// 指定マスの石を返す。
    #[inline]
    #[must_use]
    pub fn piece_at(self, square: Square) -> Option<Color> {
        let mask = square.bit();
        if self.black & mask != u64::MIN {
            Some(Color::Black)
        } else if self.white & mask != u64::MIN {
            Some(Color::White)
        } else {
            None
        }
    }

    /// 手番を返す。
    #[inline]
    #[must_use]
    pub const fn side_to_move(self) -> Color {
        self.side_to_move
    }

    /// 白石のビットボードを返す。
    #[inline]
    #[must_use]
    pub const fn white(self) -> u64 {
        self.white
    }
}

/// 反転させる石の集合を返す（全方向）。
fn flips(player: u64, opponent: u64, mv: Square) -> u64 {
    let mv_bb = mv.bit();

    flips_in_dir(player, opponent, mv_bb, shift_e)
        | flips_in_dir(player, opponent, mv_bb, shift_n)
        | flips_in_dir(player, opponent, mv_bb, shift_ne)
        | flips_in_dir(player, opponent, mv_bb, shift_nw)
        | flips_in_dir(player, opponent, mv_bb, shift_s)
        | flips_in_dir(player, opponent, mv_bb, shift_se)
        | flips_in_dir(player, opponent, mv_bb, shift_sw)
        | flips_in_dir(player, opponent, mv_bb, shift_w)
}

/// 反転させる石の集合を返す（1方向）。
fn flips_in_dir<F: Fn(u64) -> u64>(player: u64, opponent: u64, mv: u64, shift: F) -> u64 {
    let x1 = shift(mv) & opponent;
    if x1 == u64::MIN {
        return u64::MIN;
    }

    let x = spread(x1, opponent, &shift);
    if shift(x) & player != u64::MIN {
        x
    } else {
        u64::MIN
    }
}

/// 合法手の集合を返す。
fn legal_moves(player: u64, opponent: u64) -> u64 {
    let occupied = player | opponent;
    let empty = !occupied;

    moves_in_dir(player, opponent, empty, shift_e)
        | moves_in_dir(player, opponent, empty, shift_n)
        | moves_in_dir(player, opponent, empty, shift_ne)
        | moves_in_dir(player, opponent, empty, shift_nw)
        | moves_in_dir(player, opponent, empty, shift_s)
        | moves_in_dir(player, opponent, empty, shift_se)
        | moves_in_dir(player, opponent, empty, shift_sw)
        | moves_in_dir(player, opponent, empty, shift_w)
}

/// ある方向における合法手の集合を返す。
fn moves_in_dir<F: Fn(u64) -> u64>(player: u64, opponent: u64, empty: u64, shift: F) -> u64 {
    let x1 = shift(player) & opponent;
    if x1 == u64::MIN {
        return u64::MIN;
    }

    let x = spread(x1, opponent, &shift);
    shift(x) & empty
}

/// 東方向へシフトする。
#[inline]
const fn shift_e(bb: u64) -> u64 {
    (bb & !FILE_H).wrapping_shl(SHIFT_1)
}

/// 北方向へシフトする。
#[inline]
const fn shift_n(bb: u64) -> u64 {
    bb.wrapping_shl(SHIFT_8)
}

/// 北東方向へシフトする。
#[inline]
const fn shift_ne(bb: u64) -> u64 {
    (bb & !FILE_H).wrapping_shl(SHIFT_9)
}

/// 北西方向へシフトする。
#[inline]
const fn shift_nw(bb: u64) -> u64 {
    (bb & !FILE_A).wrapping_shl(SHIFT_7)
}

/// 南方向へシフトする。
#[inline]
const fn shift_s(bb: u64) -> u64 {
    bb.wrapping_shr(SHIFT_8)
}

/// 南東方向へシフトする。
#[inline]
const fn shift_se(bb: u64) -> u64 {
    (bb & !FILE_H).wrapping_shr(SHIFT_7)
}

/// 南西方向へシフトする。
#[inline]
const fn shift_sw(bb: u64) -> u64 {
    (bb & !FILE_A).wrapping_shr(SHIFT_9)
}

/// 西方向へシフトする。
#[inline]
const fn shift_w(bb: u64) -> u64 {
    (bb & !FILE_A).wrapping_shr(SHIFT_1)
}

/// Kogge-Stone法の拡張処理。
fn spread<F: Fn(u64) -> u64>(mut x: u64, opponent: u64, shift: F) -> u64 {
    for _ in u8::MIN..SPREAD_STEPS {
        x |= shift(x) & opponent;
    }
    x
}
