use super::tt::{TranspositionTable, Zobrist};

/// 探索の制限。
#[derive(Clone, Copy, Debug)]
pub(super) struct SearchLimits {
    /// 探索の最大深さ（ply）。
    max_depth: u8,
    /// 探索のノード上限。
    node_budget: u64,
}

impl SearchLimits {
    /// 探索の最大深さ（ply）を返す。
    pub(super) const fn max_depth(&self) -> u8 {
        self.max_depth
    }

    /// 探索制限を生成する。
    ///
    /// - `max_depth`: 探索の最大深さ（ply）
    /// - `node_budget`: 探索のノード上限（`u64::MAX` で無制限扱い）
    pub(super) const fn new(max_depth: u8, node_budget: u64) -> Self {
        Self {
            max_depth,
            node_budget,
        }
    }

    /// 探索のノード上限を返す。
    pub(super) const fn node_budget(&self) -> u64 {
        self.node_budget
    }
}

/// 探索統計。
#[derive(Default, Clone, Copy, Debug)]
pub(super) struct SearchStats {
    /// ベータカット等で枝刈りした回数。
    cutoffs: u64,
    /// 探索したノード数。
    nodes: u64,
    /// 置換表からのヒット回数。
    tt_hits: u64,
    /// 置換表へ保存した回数。
    tt_stores: u64,
}

impl SearchStats {
    /// 枝刈り（ベータカット等）の回数を加算する。
    pub(super) const fn inc_cutoffs(&mut self) {
        self.cutoffs = self.cutoffs.wrapping_add(1);
    }

    /// 探索ノード数を加算する。
    pub(super) const fn inc_nodes(&mut self) {
        self.nodes = self.nodes.wrapping_add(1);
    }

    /// 置換表ヒット回数を加算する。
    pub(super) const fn inc_tt_hits(&mut self) {
        self.tt_hits = self.tt_hits.wrapping_add(1);
    }

    /// 置換表保存回数を加算する。
    pub(super) const fn inc_tt_stores(&mut self) {
        self.tt_stores = self.tt_stores.wrapping_add(1);
    }

    /// 探索ノード数を返す。
    pub(super) const fn nodes(&self) -> u64 {
        self.nodes
    }

    #[cfg(test)]
    /// 置換表ヒット回数を返す（テスト用）。
    pub(super) const fn tt_hits(&self) -> u64 {
        self.tt_hits
    }
}

/// ノード上限等により探索を中断する。
#[derive(Debug, Clone, Copy)]
pub(super) struct SearchAbort;

/// 探索実行に必要な共有コンテキスト。
pub(super) struct SearchContext<'ctx> {
    /// 探索制限。
    limits: SearchLimits,
    /// 探索統計。
    stats: SearchStats,
    /// 置換表。
    tt: &'ctx mut TranspositionTable,
    /// Zobrist ハッシュ用の乱数表。
    zobrist: &'ctx Zobrist,
}

impl<'ctx> SearchContext<'ctx> {
    /// 探索制限を返す。
    pub(super) const fn limits(&self) -> SearchLimits {
        self.limits
    }

    /// 探索コンテキストを生成する。
    pub(super) fn new(
        limits: SearchLimits,
        tt: &'ctx mut TranspositionTable,
        zobrist: &'ctx Zobrist,
    ) -> Self {
        Self {
            limits,
            stats: SearchStats::default(),
            tt,
            zobrist,
        }
    }

    /// 探索統計を返す。
    pub(super) const fn stats(&self) -> SearchStats {
        self.stats
    }

    /// 探索統計への可変参照を返す。
    pub(super) const fn stats_mut(&mut self) -> &mut SearchStats {
        &mut self.stats
    }

    /// 置換表への参照を返す。
    pub(super) const fn tt(&self) -> &TranspositionTable {
        &*self.tt
    }

    /// 置換表への可変参照を返す。
    pub(super) const fn tt_mut(&mut self) -> &mut TranspositionTable {
        &mut *self.tt
    }

    /// Zobrist ハッシュ用の乱数表を返す。
    pub(super) const fn zobrist(&self) -> &'ctx Zobrist {
        self.zobrist
    }
}
