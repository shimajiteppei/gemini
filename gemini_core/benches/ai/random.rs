//! `core::ai::random` の性能計測（1手選択）。

use criterion::BatchSize;
use criterion::BenchmarkId;
use criterion::Criterion;
use criterion::black_box;
use gemini_core::ai::types::Ai;
use gemini_core::{ai, engine};

/// `cargo bench` の引数を取り込みつつ `Criterion` を生成する。
fn criterion_configured() -> Criterion {
    let base = Criterion::default();
    base.configure_from_args()
}

/// 指定手数だけ進めた局面を返す（途中で終局した場合はその時点で止める）。
fn position_after_plies(plies: u16) -> engine::Position {
    let mut black_agent = ai::random::Agent::new(u64::MIN);
    let mut game = engine::Game::initial();
    let mut white_agent = ai::random::Agent::new(u64::MIN.wrapping_add(1));

    for _turn in u16::MIN..plies {
        let position = game.position();

        let mv = match game.side_to_move() {
            engine::Color::Black => black_agent.select_move(position),
            engine::Color::White => white_agent.select_move(position),
            _ => ai::Move::Pass,
        };

        let play_result = match mv {
            ai::Move::Pass => game.play(None),
            ai::Move::Place(square) => game.play(Some(square)),
            _ => game.play(None),
        };

        let status = match play_result {
            Ok(value) => value,
            Err(_err) => break,
        };

        if let engine::GameStatus::GameOver { .. } = status {
            break;
        }
    }

    game.position()
}

/// ベンチ用に代表局面をいくつか用意する。
fn position_samples() -> [engine::Position; 3] {
    let p0 = engine::Position::initial();
    let p1 = position_after_plies(8);
    let p2 = position_after_plies(24);
    [p0, p1, p2]
}

/// `random::Agent::select_move` を計測する。
fn bench_select_move(criterion: &mut Criterion) {
    let samples = position_samples();
    let mut group = criterion.benchmark_group("ai/random/select_move");

    for (index, position) in samples.iter().enumerate() {
        let bench_id = BenchmarkId::new("pos", index);
        group.bench_with_input(bench_id, position, |bench, input| {
            bench.iter_batched(
                || ai::random::Agent::new(u64::MIN),
                |mut agent| black_box(agent.select_move(*input)),
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

/// ベンチマークのエントリーポイント。
fn main() {
    let mut criterion = criterion_configured();
    bench_select_move(&mut criterion);
    criterion.final_summary();
}
