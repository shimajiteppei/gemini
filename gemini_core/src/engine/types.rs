/// 手番（石の色）。
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum Color {
    /// 先手。
    Black,
    /// 後手。
    White,
}

impl Color {
    /// 相手側の色を返す。
    #[inline]
    #[must_use]
    pub const fn opponent(self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}

/// 盤面上のマス（0..=63のインデックス）。
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Square(
    /// `y * 8 + x` に対応する0..=63の値。
    u8,
);

impl Square {
    /// 盤の一辺の長さ。
    pub const BOARD_LEN: u8 = 8;

    /// そのマスを表すビット（`u64`）を返す。
    #[inline]
    #[must_use]
    pub fn bit(self) -> u64 {
        let one = u64::MIN.wrapping_add(1);
        let shift = u32::from(self.0);

        one.checked_shl(shift).unwrap_or(u64::MIN)
    }

    /// インデックスから `Square` を生成する（範囲チェックなし）。
    #[inline]
    pub(crate) const fn from_index_unchecked(index: u8) -> Self {
        Self(index)
    }

    /// 盤面座標（x, y）から `Square` を生成する。
    #[inline]
    #[must_use]
    pub const fn from_xy(x: u8, y: u8) -> Option<Self> {
        if x >= Self::BOARD_LEN || y >= Self::BOARD_LEN {
            return None;
        }

        let mut idx = match y.checked_mul(Self::BOARD_LEN) {
            Some(value) => value,
            None => return None,
        };

        idx = match idx.checked_add(x) {
            Some(value) => value,
            None => return None,
        };

        Some(Self(idx))
    }

    /// 0..=63 のインデックスを返す。
    #[inline]
    #[must_use]
    pub const fn index(self) -> u8 {
        self.0
    }

    /// x 座標（0..=7）を返す。
    #[inline]
    #[must_use]
    pub const fn x(self) -> u8 {
        match self.0.checked_rem(Self::BOARD_LEN) {
            Some(value) => value,
            None => u8::MIN,
        }
    }

    /// y 座標（0..=7）を返す。
    #[inline]
    #[must_use]
    pub const fn y(self) -> u8 {
        match self.0.checked_div(Self::BOARD_LEN) {
            Some(value) => value,
            None => u8::MIN,
        }
    }
}
