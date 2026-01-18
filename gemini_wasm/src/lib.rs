//! WASM (Canvas) 向けの最小 UI。
//!
//! - `wasm32` ターゲットのみで `wasm-bindgen` / `web-sys` を有効化する。
//! - それ以外のターゲットでは、workspace の `cargo test` / `cargo clippy` を通すためにスタブを提供する。

#[cfg(target_arch = "wasm32")]
mod wasm32_app {
    use gemini_core::ai::types::Ai;
    use gemini_core::{ai, engine};
    use wasm_bindgen::prelude::wasm_bindgen;

    /// 盤面の描画は JS 側に委譲する。
    ///
    /// Rust 側は「描画イベントを発火する」だけにして、Canvas API には触れない。
    #[wasm_bindgen]
    extern "C" {
        /// 1フレームの描画開始（盤面/背景のクリア等）。
        #[wasm_bindgen(js_namespace = window)]
        fn render_begin();

        /// 合法手ハイライト用。
        #[wasm_bindgen(js_namespace = window)]
        fn render_hint(x: u8, y: u8);

        /// 駒の描画。
        ///
        /// - `color`: 0=Black, 1=White
        #[wasm_bindgen(js_namespace = window)]
        fn render_cell(x: u8, y: u8, color: u8);

        /// 1フレームの描画終了（後処理があれば）。
        #[wasm_bindgen(js_namespace = window)]
        fn render_end();
    }

    const COLOR_BLACK: u8 = 0;
    const COLOR_WHITE: u8 = 1;

    const SIDE_BLACK: u8 = 0;
    const SIDE_WHITE: u8 = 1;
    const SIDE_UNKNOWN: u8 = 255;

    const STATUS_IN_PROGRESS: u8 = 0;
    const STATUS_BLACK_WINS: u8 = 1;
    const STATUS_WHITE_WINS: u8 = 2;
    const STATUS_DRAW: u8 = 3;
    const STATUS_UNKNOWN: u8 = 255;

    #[derive(Debug)]
    enum Controller {
        Alphabeta(ai::alphabeta::Agent),
        Human,
        Random(ai::random::Agent),
    }

    impl Controller {
        fn is_human(&self) -> bool {
            matches!(self, Self::Human)
        }

        fn select_move(&mut self, position: engine::Position) -> ai::Move {
            match self {
                Self::Alphabeta(agent) => agent.select_move(position),
                Self::Random(agent) => agent.select_move(position),
                Self::Human => ai::Move::Pass,
            }
        }
    }

    /// ブラウザ上で進行するアプリ状態。
    #[wasm_bindgen]
    #[derive(Debug)]
    pub struct App {
        black: Controller,
        game: engine::Game,
        white: Controller,
    }

    #[wasm_bindgen]
    impl App {
        /// human（黒） vs `depth_white` の alphabeta（白）。
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self {
                black: Controller::Human,
                game: engine::Game::initial(),
                white: Controller::Human,
            }
        }

        /// 黒を alphabeta に切り替える。
        pub fn set_black_alphabeta(&mut self, depth: u8) {
            self.black = Controller::Alphabeta(ai::alphabeta::Agent::new(depth));
        }

        /// 黒を random に切り替える。
        pub fn set_black_random(&mut self, seed: u64) {
            self.black = Controller::Random(ai::random::Agent::new(seed));
        }

        /// 黒を human に切り替える。
        pub fn set_black_human(&mut self) {
            self.black = Controller::Human;
        }

        /// 白を alphabeta に切り替える。
        pub fn set_white_alphabeta(&mut self, depth: u8) {
            self.white = Controller::Alphabeta(ai::alphabeta::Agent::new(depth));
        }

        /// 白を random に切り替える。
        pub fn set_white_random(&mut self, seed: u64) {
            self.white = Controller::Random(ai::random::Agent::new(seed));
        }

        /// 白を human に切り替える。
        pub fn set_white_human(&mut self) {
            self.white = Controller::Human;
        }

        /// クリック入力（盤面座標）。合法手なら着手し true。
        pub fn click(&mut self, x: u8, y: u8) -> bool {
            if self.game.is_game_over() {
                return false;
            }
            let side = self.game.side_to_move();
            if !self.controller_for(side).is_human() {
                return false;
            }

            let square = match engine::Square::from_xy(x, y) {
                Some(value) => value,
                None => return false,
            };

            self.game.play(Some(square)).is_ok()
        }

        /// パスを試みる（合法なら true）。
        pub fn pass(&mut self) -> bool {
            if self.game.is_game_over() {
                return false;
            }
            let side = self.game.side_to_move();
            if !self.controller_for(side).is_human() {
                return false;
            }

            self.game.play(None).is_ok()
        }

        /// AI 手番を 1 手だけ進める。
        ///
        /// - 実行した手数（0 or 1）を返す。
        /// - 300ms 遅延などの「待ち」は JS 側の責務とする。
        pub fn tick_ai(&mut self) -> u32 {
            if self.game.is_game_over() {
                return 0;
            }

            let side = self.game.side_to_move();
            if self.controller_for(side).is_human() {
                // 人間手番だが合法手が無い場合は自動パスして、ゲーム進行が止まらないようにする。
                if self.game.auto_pass_if_needed() {
                    return 1;
                }
                return 0;
            }

            let position = self.game.position();
            let mv = self.controller_for_mut(side).select_move(position);
            let play_result = match mv {
                ai::Move::Pass => self.game.play(None),
                ai::Move::Place(square) => self.game.play(Some(square)),
                _ => self.game.play(None),
            };
            if play_result.is_ok() {
                1
            } else {
                0
            }
        }

        /// AI が動けるなら最大 `max_steps` 手だけ進める。実行した手数を返す。
        pub fn tick(&mut self, max_steps: u32) -> u32 {
            let mut done: u32 = 0;
            for _ in 0..max_steps {
                if self.game.is_game_over() {
                    break;
                }

                let side = self.game.side_to_move();
                if self.controller_for(side).is_human() {
                    break;
                }

                let position = self.game.position();
                let mv = self.controller_for_mut(side).select_move(position);
                let play_result = match mv {
                    ai::Move::Pass => self.game.play(None),
                    ai::Move::Place(square) => self.game.play(Some(square)),
                    _ => self.game.play(None),
                };
                if play_result.is_ok() {
                    done = done.saturating_add(1);
                } else {
                    break;
                }
            }
            done
        }

        /// 盤面上の黒石数を返す。
        pub fn count_black(&self) -> u32 {
            let position = self.game.position();
            let (black, _) = position.counts();
            black
        }

        /// 盤面上の白石数を返す。
        pub fn count_white(&self) -> u32 {
            let position = self.game.position();
            let (_, white) = position.counts();
            white
        }

        /// 手番を返す。
        ///
        /// - 0=Black, 1=White, 255=Unknown
        pub fn side_to_move(&self) -> u8 {
            match self.game.side_to_move() {
                engine::Color::Black => SIDE_BLACK,
                engine::Color::White => SIDE_WHITE,
                _ => SIDE_UNKNOWN,
            }
        }

        /// ゲーム状態（勝敗）を返す。
        ///
        /// - 0=InProgress
        /// - 1=Black wins
        /// - 2=White wins
        /// - 3=Draw
        /// - 255=Unknown
        pub fn status_code(&self) -> u8 {
            match self.game.status() {
                engine::GameStatus::InProgress => STATUS_IN_PROGRESS,
                engine::GameStatus::GameOver { black, white } => {
                    if black > white {
                        STATUS_BLACK_WINS
                    } else if black < white {
                        STATUS_WHITE_WINS
                    } else {
                        STATUS_DRAW
                    }
                }
                _ => STATUS_UNKNOWN,
            }
        }

        /// 盤面の描画イベントを発火する（実描画は JS 側）。
        pub fn render(&self) {
            render_begin();

            let position = self.game.position();
            let legal_moves = position.legal_moves();
            let highlight = self.controller_for(self.game.side_to_move()).is_human();

            for y in 0..8 {
                for x in 0..8 {
                    let square = match engine::Square::from_xy(x, y) {
                        Some(value) => value,
                        None => continue,
                    };

                    if highlight && legal_moves & square.bit() != u64::MIN {
                        render_hint(x, y);
                    }

                    match position.piece_at(square) {
                        Some(engine::Color::Black) => render_cell(x, y, COLOR_BLACK),
                        Some(engine::Color::White) => render_cell(x, y, COLOR_WHITE),
                        _ => {}
                    }
                }
            }

            render_end();
        }
    }

    impl App {
        fn controller_for(&self, color: engine::Color) -> &Controller {
            match color {
                engine::Color::Black => &self.black,
                engine::Color::White => &self.white,
                _ => &self.black,
            }
        }

        fn controller_for_mut(&mut self, color: engine::Color) -> &mut Controller {
            match color {
                engine::Color::Black => &mut self.black,
                engine::Color::White => &mut self.white,
                _ => &mut self.black,
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm32_app::App;

#[cfg(not(target_arch = "wasm32"))]
mod non_wasm_stub {
    #[derive(Debug, Default)]
    pub struct App;

    impl App {
        pub fn new(_depth_white: u8) -> Self {
            Self
        }

        pub fn set_black_alphabeta(&mut self, _depth: u8) {}

        pub fn set_black_random(&mut self, _seed: u64) {}

        pub fn set_black_human(&mut self) {}

        pub fn set_white_random(&mut self, _seed: u64) {}

        pub fn set_white_alphabeta(&mut self, _depth: u8) {}

        pub fn set_white_human(&mut self) {}

        pub fn click(&mut self, _x: u8, _y: u8) -> bool {
            false
        }

        pub fn pass(&mut self) -> bool {
            false
        }

        pub fn tick_ai(&mut self) -> u32 {
            0
        }

        pub fn tick(&mut self, _max_steps: u32) -> u32 {
            0
        }

        pub fn count_black(&self) -> u32 {
            0
        }

        pub fn count_white(&self) -> u32 {
            0
        }

        pub fn side_to_move(&self) -> u8 {
            255
        }

        pub fn status_code(&self) -> u8 {
            255
        }

        pub fn render(&self) {}
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use non_wasm_stub::App;
