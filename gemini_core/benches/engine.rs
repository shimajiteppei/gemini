//! `core::engine` の性能計測（合法手生成、着手適用）。

use core::hint::black_box;
use criterion::BatchSize;
use criterion::Criterion;
use gemini_core::engine;

/// `cargo bench` の引数を取り込みつつ `Criterion` を生成する。
fn criterion_configured() -> Criterion {
    let base = Criterion::default();
    base.configure_from_args()
}

/// 初期局面（黒番）での代表的な合法手を返す。
const fn initial_black_move_square() -> Option<engine::Square> {
    engine::Square::from_xy(2, 3)
}

/// `Position::apply_move` を計測する。
fn bench_apply_move(criterion: &mut Criterion) {
    let square_opt = initial_black_move_square();
    let square = match square_opt {
        Some(value) => value,
        None => return,
    };

    criterion.bench_function("engine/apply_move_initial", |bench| {
        bench.iter_batched(
            engine::Position::initial,
            |position| black_box(position.apply_move(square)),
            BatchSize::SmallInput,
        );
    });
}

/// `Position::legal_moves` を計測する。
fn bench_legal_moves(criterion: &mut Criterion) {
    criterion.bench_function("engine/legal_moves_initial", |bench| {
        bench.iter(|| black_box(engine::Position::initial().legal_moves()));
    });
}

/// ベンチマークのエントリーポイント。
fn main() {
    let mut criterion = criterion_configured();

    bench_apply_move(&mut criterion);
    bench_legal_moves(&mut criterion);

    criterion.final_summary();
}
