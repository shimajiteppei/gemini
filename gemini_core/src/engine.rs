/// ゲーム進行（手番、終局判定など）の実装。
pub mod game;
/// 局面（ビットボード）と合法手/反転処理の実装。
pub mod position;
pub mod types;

pub type Position = position::Position;
pub type Game = game::Game;
pub type Color = types::Color;
pub type Square = types::Square;
pub type GameStatus = game::Status;
pub type PlayError = game::PlayError;
