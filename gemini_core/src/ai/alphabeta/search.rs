use crate::ai::types::Move;
use crate::engine::position::Position;
use crate::engine::types::Square;

use super::eval::{empty_count, evaluate, terminal_score};
#[cfg(test)]
use super::limits::SearchStats;
use super::limits::{SearchAbort, SearchContext, SearchLimits};
use super::move_ordering::{order_moves, square_from_bit};
use super::tt::{Bound, TranspositionTable, Zobrist};
use super::{ENDGAME_EMPTY_THRESHOLD, INF};

/// 探索結果。
#[derive(Clone, Copy, Debug)]
pub(super) struct SearchResult {
    /// ルートで選択した最善手。
    best_move: Move,
    /// `best_move` の評価値。
    #[cfg(test)]
    best_score: i32,
    /// 探索を完了した深さ。
    #[cfg(test)]
    completed_depth: u8,
    /// 探索統計。
    #[cfg(test)]
    stats: SearchStats,
}

impl SearchResult {
    /// ルートで選択した最善手を返す。
    pub(super) const fn best_move(self) -> Move {
        self.best_move
    }

    #[cfg(test)]
    /// `best_move` の評価値を返す（テスト用）。
    pub(super) const fn best_score(self) -> i32 {
        self.best_score
    }

    #[cfg(test)]
    /// 探索を完了した深さを返す（テスト用）。
    pub(super) const fn completed_depth(self) -> u8 {
        self.completed_depth
    }

    #[cfg(test)]
    /// 探索統計を返す（テスト用）。
    pub(super) const fn stats(self) -> SearchStats {
        self.stats
    }
}

/// 探索深さを正規化する（0の場合は1にする）。
#[inline]
pub(super) const fn normalize_depth(depth: u8) -> u8 {
    if depth == u8::MIN {
        u8::MIN.wrapping_add(1)
    } else {
        depth
    }
}

/// ルート探索（反復深化 + 終盤完全探索スイッチ）。
pub(super) fn search_root(
    position: Position,
    limits: SearchLimits,
    tt: &mut TranspositionTable,
    zobrist: &Zobrist,
) -> SearchResult {
    let legal_moves = position.legal_moves();
    if legal_moves == u64::MIN {
        return SearchResult {
            best_move: Move::Pass,
            #[cfg(test)]
            best_score: 0,
            #[cfg(test)]
            completed_depth: 0,
            #[cfg(test)]
            stats: SearchStats::default(),
        };
    }

    let empty = empty_count(position);
    if empty <= ENDGAME_EMPTY_THRESHOLD {
        // 終盤は終局まで読み切る（ノード上限は無制限にする）。
        // パスが混ざるため、残り空きマスだけで深さを固定せず、余裕のある残り手数を使う。
        let plies =
            u8::try_from(u16::from(empty).saturating_mul(2).saturating_add(2)).unwrap_or(u8::MAX);
        let exact_limits = SearchLimits::new(plies, u64::MAX);
        return endgame_root_search(position, exact_limits, tt, zobrist);
    }

    iterative_deepening(position, limits, tt, zobrist)
}

/// 反復深化によるルート探索。
fn iterative_deepening(
    position: Position,
    limits: SearchLimits,
    tt: &mut TranspositionTable,
    zobrist: &Zobrist,
) -> SearchResult {
    let fallback = first_legal_move(position);
    let mut best_move = fallback;
    #[cfg(test)]
    let mut best_score = i32::MIN;
    #[cfg(test)]
    let mut completed_depth = 0;

    let mut ctx = SearchContext::new(limits, tt, zobrist);

    for depth in 1..=limits.max_depth() {
        let result = root_search(position, depth, &mut ctx);
        match result {
            Ok((mv, score)) => {
                best_move = mv;
                #[cfg(test)]
                {
                    best_score = score;
                    completed_depth = depth;
                };
                let _: i32 = score;
            }
            Err(SearchAbort) => break,
        }
    }

    SearchResult {
        best_move,
        #[cfg(test)]
        best_score,
        #[cfg(test)]
        completed_depth,
        #[cfg(test)]
        stats: ctx.stats(),
    }
}

/// 終盤（空きマスが少ない局面）のルート探索。
fn endgame_root_search(
    position: Position,
    limits: SearchLimits,
    tt: &mut TranspositionTable,
    zobrist: &Zobrist,
) -> SearchResult {
    let fallback = first_legal_move(position);
    let mut ctx = SearchContext::new(limits, tt, zobrist);

    let depth = limits.max_depth();
    let (mv, score) = match root_search_exact(position, depth, &mut ctx) {
        Ok(value) => value,
        Err(SearchAbort) => (fallback, 0_i32),
    };
    let _: i32 = score;

    SearchResult {
        best_move: mv,
        #[cfg(test)]
        best_score: score,
        #[cfg(test)]
        completed_depth: depth,
        #[cfg(test)]
        stats: ctx.stats(),
    }
}

/// 合法手のうち1つを適当に選ぶ（合法手なしならパス）。
fn first_legal_move(position: Position) -> Move {
    let legal_moves = position.legal_moves();
    if legal_moves == u64::MIN {
        return Move::Pass;
    }

    let choice = legal_moves & legal_moves.wrapping_neg();
    let square = square_from_bit(choice).unwrap_or(Square::from_index_unchecked(0));
    Move::Place(square)
}

/// ルート探索（指定深さの探索）。
fn root_search(
    position: Position,
    depth: u8,
    ctx: &mut SearchContext<'_>,
) -> Result<(Move, i32), SearchAbort> {
    let legal_moves = position.legal_moves();
    if legal_moves == u64::MIN {
        return Ok((Move::Pass, 0_i32));
    }

    let key = ctx.zobrist().hash(position);
    let tt_move = ctx.tt().probe_best_move(key);

    let moves = order_moves(&position, legal_moves, tt_move);
    let mut best_move: Option<Square> = None;
    let mut best_score = i32::MIN;
    let mut alpha = -INF;
    let beta = INF;
    let next_depth = depth.saturating_sub(1);

    for mv in moves {
        let next = match position.apply_move(mv) {
            Ok(value) => value,
            Err(_err) => continue,
        };
        let score = match negamax(
            next,
            next_depth,
            beta.wrapping_neg(),
            alpha.wrapping_neg(),
            ctx,
        ) {
            Ok(value) => value.wrapping_neg(),
            Err(err) => return Err(err),
        };
        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }

    Ok((best_move.map_or(Move::Pass, Move::Place), best_score))
}

/// ルート探索（終局まで探索するための正確探索）。
fn root_search_exact(
    position: Position,
    depth: u8,
    ctx: &mut SearchContext<'_>,
) -> Result<(Move, i32), SearchAbort> {
    let legal_moves = position.legal_moves();
    if legal_moves == u64::MIN {
        // 終盤完全探索では、合法手なし＝パス（ただし双方パスなら終局スコア）。
        let opp = position.side_to_move().opponent();
        if position.legal_moves_for(opp) == u64::MIN {
            return Ok((Move::Pass, terminal_score(position)));
        }
        let score = match negamax_exact(
            position.pass(),
            depth.saturating_sub(1),
            INF.wrapping_neg(),
            INF,
            ctx,
        ) {
            Ok(value) => value.wrapping_neg(),
            Err(err) => return Err(err),
        };
        return Ok((Move::Pass, score));
    }

    let key = ctx.zobrist().hash(position);
    let tt_move = ctx.tt().probe_best_move(key);
    let moves = order_moves(&position, legal_moves, tt_move);

    let mut best_move: Option<Square> = None;
    let mut best_score = i32::MIN;
    let mut alpha = -INF;
    let beta = INF;
    let next_depth = depth.saturating_sub(1);

    for mv in moves {
        let next = match position.apply_move(mv) {
            Ok(value) => value,
            Err(_err) => continue,
        };
        let score = match negamax_exact(
            next,
            next_depth,
            beta.wrapping_neg(),
            alpha.wrapping_neg(),
            ctx,
        ) {
            Ok(value) => value.wrapping_neg(),
            Err(err) => return Err(err),
        };
        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }

    Ok((best_move.map_or(Move::Pass, Move::Place), best_score))
}

/// 置換表を参照し、探索窓（`alpha`/`beta`）を狭める。
///
/// - `Exact` の場合はその値を即座に返す。
/// - `Lower`/`Upper` の場合は `alpha`/`beta` を更新し、カットできるなら値を返す。
/// - 更新後に `alpha >= beta` となった場合も値を返す（カット）。
fn tt_probe_adjust_window(
    key: u64,
    depth: u8,
    alpha: &mut i32,
    beta: &mut i32,
    ctx: &mut SearchContext<'_>,
) -> Option<i32> {
    let entry = match ctx.tt().probe(key, depth) {
        Some(value) => value,
        None => return None,
    };

    ctx.stats_mut().inc_tt_hits();

    match entry.bound() {
        Bound::Exact => return Some(entry.value()),
        Bound::Lower => {
            let value = entry.value();
            if value >= *beta {
                return Some(value);
            }
            if value > *alpha {
                *alpha = value;
            }
        }
        Bound::Upper => {
            let value = entry.value();
            if value <= *alpha {
                return Some(value);
            }
            if value < *beta {
                *beta = value;
            }
        }
    }
    if *alpha >= *beta {
        return Some(entry.value());
    }
    None
}

/// ネガマックス（αβ付き、heuristic 用）。
pub(super) fn negamax(
    position: Position,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
    ctx: &mut SearchContext<'_>,
) -> Result<i32, SearchAbort> {
    ctx.stats_mut().inc_nodes();
    if ctx.stats().nodes() >= ctx.limits().node_budget() {
        return Err(SearchAbort);
    }

    let key = ctx.zobrist().hash(position);
    if let Some(value) = tt_probe_adjust_window(key, depth, &mut alpha, &mut beta, ctx) {
        return Ok(value);
    }

    let legal_moves = position.legal_moves();
    if legal_moves == u64::MIN {
        let opp = position.side_to_move().opponent();
        if position.legal_moves_for(opp) == u64::MIN {
            return Ok(terminal_score(position));
        }
        if depth == 0 {
            return Ok(evaluate(position));
        }
        let score = match negamax(
            position.pass(),
            depth.saturating_sub(1),
            beta.wrapping_neg(),
            alpha.wrapping_neg(),
            ctx,
        ) {
            Ok(value) => value.wrapping_neg(),
            Err(err) => return Err(err),
        };
        return Ok(score);
    }

    if depth == 0 {
        return Ok(evaluate(position));
    }

    let alpha_orig = alpha;
    let tt_move = ctx.tt().probe_best_move(key);
    let moves = order_moves(&position, legal_moves, tt_move);

    let next_depth = depth.saturating_sub(1);
    let mut best = i32::MIN;
    let mut best_move: Option<Square> = None;

    for mv in moves {
        let next = match position.apply_move(mv) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let score = match negamax(
            next,
            next_depth,
            beta.wrapping_neg(),
            alpha.wrapping_neg(),
            ctx,
        ) {
            Ok(value) => value.wrapping_neg(),
            Err(err) => return Err(err),
        };
        if score > best {
            best = score;
            best_move = Some(mv);
        }
        if best > alpha {
            alpha = best;
        }
        if alpha >= beta {
            ctx.stats_mut().inc_cutoffs();
            break;
        }
    }

    let bound = if best <= alpha_orig {
        Bound::Upper
    } else if best >= beta {
        Bound::Lower
    } else {
        Bound::Exact
    };

    ctx.tt_mut().store(key, depth, best, bound, best_move);
    ctx.stats_mut().inc_tt_stores();

    Ok(best)
}

/// ネガマックス（αβ付き、終盤完全探索用）。
pub(super) fn negamax_exact(
    position: Position,
    depth: u8,
    mut alpha: i32,
    mut beta: i32,
    ctx: &mut SearchContext<'_>,
) -> Result<i32, SearchAbort> {
    ctx.stats_mut().inc_nodes();
    if ctx.stats().nodes() >= ctx.limits().node_budget() {
        return Err(SearchAbort);
    }

    let key = ctx.zobrist().hash(position);
    if let Some(value) = tt_probe_adjust_window(key, depth, &mut alpha, &mut beta, ctx) {
        return Ok(value);
    }

    let legal_moves = position.legal_moves();
    if legal_moves == u64::MIN {
        let opp = position.side_to_move().opponent();
        if position.legal_moves_for(opp) == u64::MIN {
            return Ok(terminal_score(position));
        }
        if depth == 0 {
            // 深さが尽きるケースは想定外だが、最悪でも終局スコアを返す。
            return Ok(terminal_score(position));
        }
        let score = match negamax_exact(
            position.pass(),
            depth.saturating_sub(1),
            beta.wrapping_neg(),
            alpha.wrapping_neg(),
            ctx,
        ) {
            Ok(value) => value.wrapping_neg(),
            Err(err) => return Err(err),
        };
        return Ok(score);
    }

    if depth == 0 {
        // 深さが尽きるケースは想定外だが、最悪でも終局スコアを返す。
        return Ok(terminal_score(position));
    }

    let alpha_orig = alpha;
    let tt_move = ctx.tt().probe_best_move(key);
    let moves = order_moves(&position, legal_moves, tt_move);

    let next_depth = depth.saturating_sub(1);
    let mut best = i32::MIN;
    let mut best_move: Option<Square> = None;

    for mv in moves {
        let next = match position.apply_move(mv) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let score = match negamax_exact(
            next,
            next_depth,
            beta.wrapping_neg(),
            alpha.wrapping_neg(),
            ctx,
        ) {
            Ok(value) => value.wrapping_neg(),
            Err(err) => return Err(err),
        };
        if score > best {
            best = score;
            best_move = Some(mv);
        }
        if best > alpha {
            alpha = best;
        }
        if alpha >= beta {
            ctx.stats_mut().inc_cutoffs();
            break;
        }
    }

    let bound = if best <= alpha_orig {
        Bound::Upper
    } else if best >= beta {
        Bound::Lower
    } else {
        Bound::Exact
    };

    ctx.tt_mut().store(key, depth, best, bound, best_move);
    ctx.stats_mut().inc_tt_stores();

    Ok(best)
}
