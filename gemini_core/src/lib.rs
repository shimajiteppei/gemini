//! Reversi (Othello) core logic.
//!
//! このクレートはゲーム進行を管理する `engine` と、手を選択する `ai` を提供します。
//! UI（`sdl` / `wasm`）から利用されることを想定しています。

#![forbid(unsafe_code)]

/// ゲームルール・局面・進行を提供するモジュール。
pub mod engine;

/// AI（手選択アルゴリズム）を提供するモジュール。
pub mod ai;
