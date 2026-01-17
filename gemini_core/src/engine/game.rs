use crate::engine::position::{ApplyMoveError, Position};
use crate::engine::types::{Color, Square};

/// ゲームの状態。
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Status {
    /// 終局（双方がパス）。
    GameOver {
        /// 黒の石数。
        black: u32,
        /// 白の石数。
        white: u32,
    },
    /// 進行中。
    InProgress,
}

/// 手の適用（打つ/パス）に失敗した理由。
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PlayError {
    /// すでに終局している。
    GameOver,
    /// 指定マスが合法手ではない。
    IllegalMove,
    /// 合法手があるのにパスしようとした。
    PassNotAllowed,
}

/// 1ゲームの進行を管理する構造体。
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Game {
    /// 連続パス回数。
    consecutive_passes: u8,
    /// 現在の局面。
    position: Position,
}

impl Game {
    /// 初期局面からゲームを開始する。
    #[inline]
    #[must_use]
    pub const fn initial() -> Self {
        Self {
            consecutive_passes: u8::MIN,
            position: Position::initial(),
        }
    }

    /// 終局しているかどうかを返す。
    #[inline]
    #[must_use]
    pub fn is_game_over(self) -> bool {
        if self.consecutive_passes >= 2 {
            return true;
        }

        if self.position.can_play_for(self.position.side_to_move()) {
            return false;
        }

        !self
            .position
            .can_play_for(self.position.side_to_move().opponent())
    }

    /// 1手（打つ/パス）を適用する。
    ///
    /// # Errors
    ///
    /// 次の場合にエラーを返す：
    /// - `PlayError::GameOver`: すでにゲームが終局している場合
    /// - `PlayError::IllegalMove`: 指定されたマスが合法手でない場合
    /// - `PlayError::PassNotAllowed`: 合法手が存在するのにパスを試みた場合
    ///
    #[inline]
    pub fn play(&mut self, mv: Option<Square>) -> Result<Status, PlayError> {
        if self.is_game_over() {
            return Err(PlayError::GameOver);
        }

        if let Some(square) = mv {
            let next = match self.position.apply_move(square) {
                Ok(next_position) => next_position,
                Err(err) => {
                    return Err(match err {
                        ApplyMoveError::IllegalMove => PlayError::IllegalMove,
                    });
                }
            };

            self.consecutive_passes = u8::MIN;
            self.position = next;
        } else {
            if self.position.legal_moves() != u64::MIN {
                return Err(PlayError::PassNotAllowed);
            }

            self.consecutive_passes = self.consecutive_passes.saturating_add(1);
            self.position = self.position.pass();
        }

        Ok(self.status())
    }

    /// 現在の局面を返す。
    #[inline]
    #[must_use]
    pub const fn position(self) -> Position {
        self.position
    }

    /// 現手番を返す。
    #[inline]
    #[must_use]
    pub const fn side_to_move(self) -> Color {
        self.position.side_to_move()
    }

    /// 現在のゲーム状態を返す。
    #[inline]
    #[must_use]
    pub fn status(self) -> Status {
        if self.is_game_over() {
            let (black, white) = self.position.counts();
            return Status::GameOver { black, white };
        }

        Status::InProgress
    }
}
