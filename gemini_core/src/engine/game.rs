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
    /// 現手番に合法手が無い場合、パスを自動適用する。
    ///
    /// UI 側で「人間が打てない状態で入力待ちのまま止まる」ことを避けるための補助。
    ///
    /// - すでに終局している場合は何もしない（`false`）。
    /// - 合法手がある場合は何もしない（`false`）。
    /// - パスを適用できた場合は `true`。
    #[inline]
    pub fn auto_pass_if_needed(&mut self) -> bool {
        if self.is_game_over() {
            return false;
        }

        if self.position.legal_moves() != u64::MIN {
            return false;
        }

        self.play(None).is_ok()
    }

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

#[cfg(test)]
mod tests {
    use super::Game;
    use crate::ai::random;
    use crate::ai::types::Ai as _;
    use crate::ai::types::Move;
    use crate::engine::position::Position;

    fn find_position_where_current_player_must_pass() -> Option<Position> {
        // 決定的に見つかるまで seed を変えつつ探索する。
        for seed in 0_u64..256 {
            let mut agent = random::Agent::new(seed);
            let mut pos = Position::initial();

            // 最大 60 手程度で終局するので余裕を持たせる。
            for _ply in 0_u16..100 {
                let side = pos.side_to_move();
                let my_moves = pos.legal_moves_for(side);
                let opp_moves = pos.legal_moves_for(side.opponent());

                // 相手は打てるが自分は打てない（＝パスが必要）。
                if my_moves == u64::MIN && opp_moves != u64::MIN {
                    return Some(pos);
                }

                // 終局（双方パス）に到達したらこの seed は諦める。
                if my_moves == u64::MIN && opp_moves == u64::MIN {
                    break;
                }

                let mv = agent.select_move(pos);
                let next = match mv {
                    Move::Pass => Ok(pos.pass()),
                    Move::Place(square) => pos.apply_move(square),
                };
                pos = match next {
                    Ok(value) => value,
                    Err(_err) => break,
                };
            }
        }

        None
    }

    #[test]
    fn auto_pass_if_needed_switches_turn_when_no_legal_moves() {
        let pos_opt = find_position_where_current_player_must_pass();
        assert!(pos_opt.is_some(), "pass position not found in deterministic search");
        let pos = pos_opt.unwrap_or_else(Position::initial);

        let side_before = pos.side_to_move();
        assert_eq!(pos.legal_moves(), u64::MIN);
        assert_ne!(pos.legal_moves_for(side_before.opponent()), u64::MIN);

        let mut game = Game {
            consecutive_passes: 0,
            position: pos,
        };

        assert!(!game.is_game_over());
        assert!(game.auto_pass_if_needed());
        assert_eq!(game.side_to_move(), side_before.opponent());
        assert_ne!(game.position().legal_moves(), u64::MIN);
        assert!(!game.is_game_over());
    }
}
