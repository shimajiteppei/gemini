use crate::ai::types::{Ai, Move};
use crate::engine::position::Position;
use crate::engine::types::Square;

/// 64-bit 線形合同法 (LCG) の簡易 RNG。
/// - rand クレート不使用
/// - `seed` で決定的に再現可能
#[derive(Debug, Clone, Copy)]
struct Lcg64 {
    /// 内部状態。
    state: u64,
}

impl Lcg64 {
    /// LCG の内部状態を `seed` から初期化する。
    #[inline]
    const fn new(seed: u64) -> Self {
        // seed が 0 でも動くように軽く攪拌（任意）
        Self {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        }
    }

    /// 次の u32 を生成する（上位 32bit を返す）。
    #[inline]
    fn next_u32(&mut self) -> u32 {
        // 2^64 mod の LCG: state = state * A + C
        // よく使われる定数（PCG 系で採用される LCG 定数）
        const LCG_MULTIPLIER: u64 = 6_364_136_223_846_793_005;
        const LCG_INCREMENT: u64 = 1_442_695_040_888_963_407;

        self.state = self
            .state
            .wrapping_mul(LCG_MULTIPLIER)
            .wrapping_add(LCG_INCREMENT);

        u32::try_from(self.state >> 32).unwrap_or(u32::MAX)
    }
}

/// 合法手からランダムに1手を選択するAI。
#[derive(Debug)]
#[non_exhaustive]
pub struct Agent {
    /// 乱数生成器。
    rng: Lcg64,
}

impl Agent {
    /// `seed` を用いて初期化する。
    #[inline]
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self {
            rng: Lcg64::new(seed),
        }
    }
}

impl Ai for Agent {
    #[inline]
    fn select_move(&mut self, position: Position) -> Move {
        let moves = position.legal_moves();
        if moves == u64::MIN {
            return Move::Pass;
        }

        let choice = choose_bit(moves, self.rng.next_u32());
        let index = match u8::try_from(choice.trailing_zeros()) {
            Ok(value) => value,
            Err(_conversion_error) => return Move::Pass,
        };

        Move::Place(Square::from_index_unchecked(index))
    }
}

/// `bits` に立っているビットのうち、`random` に基づき1つ選択して返す。
fn choose_bit(bits: u64, random: u32) -> u64 {
    let count = bits.count_ones();
    if count == u32::MIN {
        return u64::MIN;
    }

    let random_u64 = u64::from(random);
    let count_u64 = u64::from(count);
    let product = random_u64.wrapping_mul(count_u64);
    let high_u64 = product.wrapping_shr(32);
    let skip = u32::try_from(high_u64).unwrap_or(u32::MAX);
    let mut bb = bits;

    for _ in u32::MIN..skip {
        bb &= bb.wrapping_sub(1);
    }

    bb & bb.wrapping_neg()
}
