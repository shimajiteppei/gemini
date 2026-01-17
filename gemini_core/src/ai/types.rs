use crate::engine::position::Position;
use crate::engine::types::Square;

/// AIが選択する手。
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Move {
    /// パス。
    Pass,
    /// 指定マスへ着手。
    Place(Square),
}

/// 手を選択するAI。
pub trait Ai {
    /// 現在局面から次の手を選択する。
    fn select_move(&mut self, position: Position) -> Move;
}
