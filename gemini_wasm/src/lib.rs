//! WASM (Canvas) 向けの最小 UI。
//!
//! - `wasm32` ターゲットのみで `wasm-bindgen` / `web-sys` を有効化する。
//! - それ以外のターゲットでは、workspace の `cargo test` / `cargo clippy` を通すためにスタブを提供する。

#[cfg(target_arch = "wasm32")]
mod wasm32_app {
    use gemini_core::ai::types::Ai;
    use gemini_core::{ai, engine};
    use wasm_bindgen::JsValue;
    use wasm_bindgen::prelude::*;
    use web_sys::CanvasRenderingContext2d;

    /// 盤面描画のオフセット。
    const OFFSET: f64 = 8.0;

    /// AI 手番の遅延（ミリ秒）。
    const AI_DELAY_MS: f64 = 300.0;

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
        next_ai_due_ms: f64,
        white: Controller,
    }

    #[wasm_bindgen]
    impl App {
        /// human（黒） vs `depth_white` の alphabeta（白）。
        #[wasm_bindgen(constructor)]
        pub fn new(depth_white: u8) -> Self {
            Self {
                black: Controller::Human,
                game: engine::Game::initial(),
                next_ai_due_ms: -1.0,
                white: Controller::Alphabeta(ai::alphabeta::Agent::new(depth_white)),
            }
        }

        /// 黒を random に切り替える。
        pub fn set_black_random(&mut self, seed: u64) {
            self.black = Controller::Random(ai::random::Agent::new(seed));
        }

        /// 黒を human に切り替える。
        pub fn set_black_human(&mut self) {
            self.black = Controller::Human;
        }

        /// 白を random に切り替える。
        pub fn set_white_random(&mut self, seed: u64) {
            self.white = Controller::Random(ai::random::Agent::new(seed));
        }

        /// 白を alphabeta に切り替える。
        pub fn set_white_alphabeta(&mut self, depth: u8) {
            self.white = Controller::Alphabeta(ai::alphabeta::Agent::new(depth));
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

            let ok = self.game.play(Some(square)).is_ok();
            if ok {
                // 次に AI 手番が来た場合は、初回 tick で遅延を開始する。
                self.next_ai_due_ms = -1.0;
            }
            ok
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

            let ok = self.game.play(None).is_ok();
            if ok {
                // 次に AI 手番が来た場合は、初回 tick で遅延を開始する。
                self.next_ai_due_ms = -1.0;
            }
            ok
        }

        /// AI 手番を 1 手だけ進める（AI 手番は 0.3 秒遅延）。
        ///
        /// - `now_ms`: `performance.now()` 相当の単調増加時刻（ミリ秒）。
        /// - 実行した手数（0 or 1）を返す。
        pub fn tick_ai(&mut self, now_ms: f64) -> u32 {
            if self.game.is_game_over() {
                return 0;
            }

            let side = self.game.side_to_move();
            if self.controller_for(side).is_human() {
                return 0;
            }

            // AI 手番に入った直後は、まず遅延タイマーをセットして待つ。
            if self.next_ai_due_ms < 0.0 {
                self.next_ai_due_ms = now_ms + AI_DELAY_MS;
                return 0;
            }
            if now_ms < self.next_ai_due_ms {
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
                // 次の AI 手番のために、遅延開始をリセットする。
                self.next_ai_due_ms = -1.0;
                1
            } else {
                0
            }
        }

        /// AI 手番遅延を今から開始する（例: 人間が着手した直後に呼ぶ）。
        pub fn arm_ai_delay(&mut self, now_ms: f64) {
            self.next_ai_due_ms = now_ms + AI_DELAY_MS;
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

        /// 状態表示用の文字列を返す。
        pub fn status_text(&self) -> String {
            let position = self.game.position();
            let (black, white) = position.counts();
            let side = self.game.side_to_move();
            let side_text = match side {
                engine::Color::Black => "Black",
                engine::Color::White => "White",
                _ => "Unknown",
            };

            let status = self.game.status();
            match status {
                engine::GameStatus::InProgress => {
                    format!("{side_text} to move | B={black} W={white}")
                }
                engine::GameStatus::GameOver { black: b, white: w } => {
                    let result = if b > w {
                        "Black wins"
                    } else if b < w {
                        "White wins"
                    } else {
                        "Draw"
                    };
                    format!("Game Over: {result} | B={b} W={w}")
                }
                _ => format!("Unknown status | B={black} W={white}"),
            }
        }

        /// Canvas へ盤面を描画する。
        ///
        /// - `cell_size`: 1マスのピクセルサイズ（例: 64.0）
        pub fn render(&self, ctx: &CanvasRenderingContext2d, cell_size: f64) {
            let board_len: f64 = 8.0;
            let board_px = board_len * cell_size;
            let full = board_px + OFFSET * 2.0;

            ctx.set_fill_style(&JsValue::from_str("#105010"));
            ctx.fill_rect(0.0, 0.0, full, full);

            let position = self.game.position();
            let legal_moves = position.legal_moves();
            let highlight = self.controller_for(self.game.side_to_move()).is_human();

            for y in 0..8 {
                for x in 0..8 {
                    let fx = f64::from(x);
                    let fy = f64::from(y);
                    let left = OFFSET + fx * cell_size;
                    let top = OFFSET + fy * cell_size;

                    ctx.set_fill_style(&JsValue::from_str("#008000"));
                    ctx.fill_rect(left, top, cell_size, cell_size);

                    ctx.set_stroke_style(&JsValue::from_str("#000000"));
                    ctx.stroke_rect(left, top, cell_size, cell_size);

                    let square = match engine::Square::from_xy(x, y) {
                        Some(value) => value,
                        None => continue,
                    };

                    if highlight && legal_moves & square.bit() != u64::MIN {
                        let r = cell_size / 10.0;
                        let cx = left + cell_size / 2.0;
                        let cy = top + cell_size / 2.0;
                        ctx.begin_path();
                        let _: Result<(), JsValue> = ctx.arc(cx, cy, r, 0.0, 6.283185307179586);
                        ctx.set_fill_style(&JsValue::from_str("#e0e040"));
                        ctx.fill();
                    }

                    let piece = position.piece_at(square);
                    let (fill, present) = match piece {
                        Some(engine::Color::Black) => ("#000000", true),
                        Some(engine::Color::White) => ("#f0f0f0", true),
                        None => ("#000000", false),
                        Some(_) => ("#808080", true),
                    };
                    if present {
                        let r = cell_size * 0.40;
                        let cx = left + cell_size / 2.0;
                        let cy = top + cell_size / 2.0;
                        ctx.begin_path();
                        let _: Result<(), JsValue> = ctx.arc(cx, cy, r, 0.0, 6.283185307179586);
                        ctx.set_fill_style(&JsValue::from_str(fill));
                        ctx.fill();
                    }
                }
            }
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

        pub fn tick_ai(&mut self, _now_ms: f64) -> u32 {
            0
        }

        pub fn arm_ai_delay(&mut self, _now_ms: f64) {}

        pub fn tick(&mut self, _max_steps: u32) -> u32 {
            0
        }

        pub fn status_text(&self) -> String {
            "wasm App is available only on wasm32".to_string()
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub use non_wasm_stub::App;
