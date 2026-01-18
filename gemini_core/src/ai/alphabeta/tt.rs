use crate::engine::position::Position;
use crate::engine::types::{Color, Square};

/// 置換表の bound 種別。
#[derive(Copy, Clone, Debug)]
pub(super) enum Bound {
    /// 正確な値。
    Exact,
    /// 下限（この値以上）。
    Lower,
    /// 上限（この値以下）。
    Upper,
}

/// 置換表エントリ。
#[derive(Copy, Clone, Debug)]
pub(super) struct TTEntry {
    /// ベストムーブ。
    best_move: Option<Square>,
    /// `value` の意味（exact/lower/upper）。
    bound: Bound,
    /// この値が保証される探索深さ。
    depth: u8,
    /// 盤面ハッシュ。
    key: u64,
    /// 評価値。
    value: i32,
}

impl TTEntry {
    /// このエントリの bound 種別を返す。
    pub(super) const fn bound(&self) -> Bound {
        self.bound
    }

    /// このエントリに保存されている評価値を返す。
    pub(super) const fn value(&self) -> i32 {
        self.value
    }
}

/// 置換表（簡易固定長）。
#[derive(Debug)]
pub(super) struct TranspositionTable {
    /// ハッシュ表本体。
    entries: Vec<TTEntry>,
}

impl TranspositionTable {
    /// キーからインデックスを求める。
    fn index(&self, key: u64) -> usize {
        let mask = self.entries.len().wrapping_sub(1);
        // wasm32 等の 32-bit 環境でも安定するよう、下位 32bit を利用して折り畳む。
        let folded = key ^ key.wrapping_shr(32);
        let low_u64 = folded & u64::from(u32::MAX);
        let low_u32 = u32::try_from(low_u64).unwrap_or(u32::MAX);
        let low_usize = match usize::try_from(low_u32) {
            Ok(value) => value,
            Err(_err) => usize::MAX,
        };
        low_usize & mask
    }

    /// 置換表を初期化する。
    pub(super) fn new(size: usize) -> Self {
        let size_pow2 = size.next_power_of_two().max(1);
        let empty = TTEntry {
            best_move: None,
            bound: Bound::Exact,
            depth: 0,
            key: 0,
            value: 0,
        };
        Self {
            entries: vec![empty; size_pow2],
        }
    }

    /// 指定深さ以上のエントリを取得する。
    pub(super) fn probe(&self, key: u64, depth: u8) -> Option<TTEntry> {
        let idx = self.index(key);
        let entry = match self.entries.get(idx) {
            Some(value) => *value,
            None => return None,
        };
        (entry.key == key && entry.depth >= depth).then_some(entry)
    }

    /// ベストムーブのみを取得する。
    pub(super) fn probe_best_move(&self, key: u64) -> Option<Square> {
        let idx = self.index(key);
        let entry = match self.entries.get(idx) {
            Some(value) => *value,
            None => return None,
        };
        if entry.key == key {
            entry.best_move
        } else {
            None
        }
    }

    /// エントリを保存する。
    pub(super) fn store(
        &mut self,
        key: u64,
        depth: u8,
        stored_value: i32,
        bound: Bound,
        best_move: Option<Square>,
    ) {
        let idx = self.index(key);
        let old = match self.entries.get(idx) {
            Some(value) => *value,
            None => return,
        };
        if old.key != key || depth >= old.depth {
            let slot = match self.entries.get_mut(idx) {
                Some(value) => value,
                None => return,
            };
            *slot = TTEntry {
                best_move,
                bound,
                depth,
                key,
                value: stored_value,
            };
        }
    }
}

/// Zobrist ハッシュ。
#[derive(Debug, Clone)]
pub(super) struct Zobrist {
    /// 黒石用乱数。
    black: [u64; 64],
    /// 手番用乱数。
    side_to_move: u64,
    /// 白石用乱数。
    white: [u64; 64],
}

impl Zobrist {
    /// 盤面をハッシュ化する。
    pub(super) fn hash(&self, position: Position) -> u64 {
        let mut key: u64 = 0;
        let mut bb = position.black();
        while bb != u64::MIN {
            let bit = bb & bb.wrapping_neg();
            let idx_u32 = bit.trailing_zeros();
            let idx = match usize::try_from(idx_u32) {
                Ok(value) => value,
                Err(_err) => {
                    bb &= bb.wrapping_sub(1);
                    continue;
                }
            };
            if let Some(value) = self.black.get(idx) {
                key ^= *value;
            }
            bb &= bb.wrapping_sub(1);
        }

        bb = position.white();
        while bb != u64::MIN {
            let bit = bb & bb.wrapping_neg();
            let idx_u32 = bit.trailing_zeros();
            let idx = match usize::try_from(idx_u32) {
                Ok(value) => value,
                Err(_err) => {
                    bb &= bb.wrapping_sub(1);
                    continue;
                }
            };
            if let Some(value) = self.white.get(idx) {
                key ^= *value;
            }
            bb &= bb.wrapping_sub(1);
        }

        // black-to-move のときだけ XOR（約束）
        if position.side_to_move() == Color::Black {
            key ^= self.side_to_move;
        }
        key
    }

    /// Zobrist テーブルを生成する。
    pub(super) fn new() -> Self {
        let mut seed: u64 = 0xDEAD_BEEF_CAFE_BABE;
        let mut black = [0_u64; 64];
        let mut white = [0_u64; 64];
        for i in u8::MIN..64_u8 {
            let idx = usize::from(i);
            if let Some(slot) = black.get_mut(idx) {
                *slot = splitmix64(&mut seed);
            }
            if let Some(slot) = white.get_mut(idx) {
                *slot = splitmix64(&mut seed);
            }
        }
        let side_to_move = splitmix64(&mut seed);
        Self {
            black,
            side_to_move,
            white,
        }
    }
}

/// `SplitMix64` による擬似乱数生成。
///
/// Zobrist テーブル初期化用の乱数列を得るために利用する。
const fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
