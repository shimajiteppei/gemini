/// 評価関数。
mod eval;
/// 探索制限・統計・コンテキスト。
mod limits;
/// 手の並べ替え。
mod move_ordering;
/// 探索本体。
mod search;
/// 置換表とZobrist。
mod tt;

#[cfg(test)]
mod tests;

use crate::ai::types::{Ai, Move};
use crate::engine::position::Position;

use limits::SearchLimits;
use search::{normalize_depth, search_root};
use tt::{TranspositionTable, Zobrist};

/// 角マス（4隅）のマスク。
const CORNER_MASK: u64 = 0x8100_0000_0000_0081;

/// 終局スコアに用いる石差スケール。
///
/// - 手番視点の石差（my - opp）に掛ける
/// - 例: 1 石差で 100 点
const DISC_SCALE: i32 = 100;

/// 終盤での完全探索へ切り替える空きマス閾値。
const ENDGAME_EMPTY_THRESHOLD: u8 = 14;

/// 探索で扱う十分大きな値。
const INF: i32 = 1_000_000_000;

/// デフォルトのノード上限（反復深化全体での訪問ノード数）。
const DEFAULT_NODE_BUDGET: u64 = 250_000;

/// 置換表のデフォルトサイズ（エントリ数、2 の冪）。
const DEFAULT_TT_SIZE: usize = 1 << 16;

/// アルファベータ探索を行うAI。
#[derive(Debug)]
#[non_exhaustive]
pub struct Agent {
    /// 探索深さ。
    depth: u8,

    /// ノード上限。
    node_budget: u64,

    /// 置換表。
    tt: TranspositionTable,

    /// Zobrist ハッシュ。
    zobrist: Zobrist,
}

impl Agent {
    /// 探索深さを返す。
    #[inline]
    #[must_use]
    pub const fn depth(&self) -> u8 {
        self.depth
    }

    /// `depth` を指定して初期化する。
    #[inline]
    #[must_use]
    pub fn new(depth: u8) -> Self {
        Self {
            depth,
            node_budget: DEFAULT_NODE_BUDGET,
            tt: TranspositionTable::new(DEFAULT_TT_SIZE),
            zobrist: Zobrist::new(),
        }
    }

    /// ノード上限を設定する。
    #[inline]
    pub const fn set_node_budget(&mut self, node_budget: u64) {
        self.node_budget = node_budget;
    }
}

impl Ai for Agent {
    #[inline]
    fn select_move(&mut self, position: Position) -> Move {
        let limits = SearchLimits::new(normalize_depth(self.depth), self.node_budget);

        let result = search_root(position, limits, &mut self.tt, &self.zobrist);
        result.best_move()
    }
}
