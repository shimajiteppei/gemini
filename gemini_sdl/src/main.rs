//! SDL で動作する最小 UI。

use gemini_core::ai::types::Ai;
use gemini_core::{ai, engine};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color as SdlColor;
use sdl2::rect::Rect;
use std::time::Duration;

/// 盤面のオフセット（左上）。
const OFFSET: i32 = 16;

/// 1マスのピクセルサイズ。
const CELL_SIZE: i32 = 64;

/// 盤面の一辺の長さ（マス）。
const BOARD_LEN: i32 = 8;

/// 盤面の一辺の長さ（ピクセル）。
const BOARD_PX: i32 = BOARD_LEN * CELL_SIZE;

/// ウィンドウ幅（ピクセル）。
const WINDOW_W: u32 = (OFFSET + BOARD_PX + OFFSET) as u32;

/// ウィンドウ高さ（ピクセル）。
const WINDOW_H: u32 = (OFFSET + BOARD_PX + OFFSET) as u32;

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

#[derive(Debug)]
struct App {
    black: Controller,
    game: engine::Game,
    white: Controller,
}

impl App {
    fn new() -> Self {
        Self {
            black: Controller::Human,
            game: engine::Game::initial(),
            white: Controller::Alphabeta(ai::alphabeta::Agent::new(3)),
        }
    }

    fn controller_for_mut(&mut self, color: engine::Color) -> &mut Controller {
        match color {
            engine::Color::Black => &mut self.black,
            engine::Color::White => &mut self.white,
            _ => &mut self.black,
        }
    }

    fn controller_for(&self, color: engine::Color) -> &Controller {
        match color {
            engine::Color::Black => &self.black,
            engine::Color::White => &self.white,
            _ => &self.black,
        }
    }

    fn status_text(&self) -> String {
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

    fn try_play(&mut self, mv: ai::Move) {
        let play_result = match mv {
            ai::Move::Pass => self.game.play(None),
            ai::Move::Place(square) => self.game.play(Some(square)),
            _ => self.game.play(None),
        };
        let _: Result<engine::GameStatus, engine::PlayError> = play_result;
    }

    fn step_ai_once(&mut self) {
        if self.game.is_game_over() {
            return;
        }

        let side = self.game.side_to_move();
        let is_human = self.controller_for(side).is_human();
        if is_human {
            return;
        }

        let position = self.game.position();
        let mv = self.controller_for_mut(side).select_move(position);
        self.try_play(mv);
    }

    fn try_human_click(&mut self, x: i32, y: i32) -> bool {
        if self.game.is_game_over() {
            return false;
        }

        let side = self.game.side_to_move();
        if !self.controller_for(side).is_human() {
            return false;
        }

        let file = x - OFFSET;
        let rank = y - OFFSET;
        if file < 0 || rank < 0 {
            return false;
        }

        let xx = file / CELL_SIZE;
        let yy = rank / CELL_SIZE;
        if !(0..BOARD_LEN).contains(&xx) || !(0..BOARD_LEN).contains(&yy) {
            return false;
        }

        let x_u8 = match u8::try_from(xx) {
            Ok(value) => value,
            Err(_err) => return false,
        };
        let y_u8 = match u8::try_from(yy) {
            Ok(value) => value,
            Err(_err) => return false,
        };

        let square = match engine::Square::from_xy(x_u8, y_u8) {
            Some(value) => value,
            None => return false,
        };

        let play_result = self.game.play(Some(square));
        play_result.is_ok()
    }

    fn try_human_pass(&mut self) -> bool {
        if self.game.is_game_over() {
            return false;
        }

        let side = self.game.side_to_move();
        if !self.controller_for(side).is_human() {
            return false;
        }

        let play_result = self.game.play(None);
        play_result.is_ok()
    }
}

fn draw_board(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, app: &App) {
    let position = app.game.position();
    let legal_moves = position.legal_moves();
    let highlight = app.controller_for(app.game.side_to_move()).is_human();

    canvas.set_draw_color(SdlColor::RGB(16, 96, 16));
    canvas.clear();

    // マス。
    for y in 0..BOARD_LEN {
        for x in 0..BOARD_LEN {
            let xx = OFFSET + x * CELL_SIZE;
            let yy = OFFSET + y * CELL_SIZE;
            let rect = Rect::new(xx, yy, CELL_SIZE as u32, CELL_SIZE as u32);

            canvas.set_draw_color(SdlColor::RGB(0, 128, 0));
            let _: Result<(), String> = canvas.fill_rect(rect);

            canvas.set_draw_color(SdlColor::RGB(0, 0, 0));
            let _: Result<(), String> = canvas.draw_rect(rect);

            let x_u8 = match u8::try_from(x) {
                Ok(value) => value,
                Err(_err) => continue,
            };
            let y_u8 = match u8::try_from(y) {
                Ok(value) => value,
                Err(_err) => continue,
            };
            let square = match engine::Square::from_xy(x_u8, y_u8) {
                Some(value) => value,
                None => continue,
            };

            if highlight && legal_moves & square.bit() != u64::MIN {
                let inset = CELL_SIZE / 3;
                let hint_rect = Rect::new(
                    xx + inset,
                    yy + inset,
                    (CELL_SIZE - inset * 2) as u32,
                    (CELL_SIZE - inset * 2) as u32,
                );
                canvas.set_draw_color(SdlColor::RGB(224, 224, 64));
                let _: Result<(), String> = canvas.fill_rect(hint_rect);
            }

            // 石。
            let piece = position.piece_at(square);
            let (color, present) = match piece {
                Some(engine::Color::Black) => (SdlColor::RGB(0, 0, 0), true),
                Some(engine::Color::White) => (SdlColor::RGB(240, 240, 240), true),
                Some(_) => (SdlColor::RGB(0, 0, 0), false),
                None => (SdlColor::RGB(0, 0, 0), false),
            };
            if present {
                let inset = CELL_SIZE / 8;
                let stone_rect = Rect::new(
                    xx + inset,
                    yy + inset,
                    (CELL_SIZE - inset * 2) as u32,
                    (CELL_SIZE - inset * 2) as u32,
                );
                canvas.set_draw_color(color);
                let _: Result<(), String> = canvas.fill_rect(stone_rect);
            }
        }
    }
}

fn main() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;

    let window = video
        .window("gemini (Reversi)", WINDOW_W, WINDOW_H)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;

    let mut app = App::new();
    let mut event_pump = sdl.event_pump()?;

    let draw_and_present = |canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, app: &App| {
        let title = app.status_text();
        let _ = canvas.window_mut().set_title(&title);
        draw_board(canvas, app);
        canvas.present();
    };

    'running: loop {
        let mut did_human_move = false;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::P),
                    ..
                } => did_human_move |= app.try_human_pass(),
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => did_human_move |= app.try_human_click(x, y),
                _ => {}
            }
        }

        if did_human_move {
            // 人間の手を打った直後に一度描画更新する。
            draw_and_present(&mut canvas, &app);

            // その後に少し待ってからAIが手を打ち、再度描画更新する。
            if !app.game.is_game_over() {
                let side = app.game.side_to_move();
                if !app.controller_for(side).is_human() {
                    std::thread::sleep(Duration::from_millis(300));
                    app.step_ai_once();
                }
            }
        } else {
            app.step_ai_once();
        }

        draw_and_present(&mut canvas, &app);
    }

    Ok(())
}
