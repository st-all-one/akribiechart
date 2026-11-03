//! Gerador determinístico (mulberry32) e semente por id.

use super::Ink;
use crate::geometry::Point;

/// Gerador mulberry32: 32 bits, sem dependência e com sequência fixa por semente.
///
/// `frozen` devolve sempre deslocamento zero — é o modo exato (sem perturbação).
#[derive(Debug, Clone, Copy)]
pub struct Rng {
    state: u32,
    frozen: bool,
}

impl Rng {
    /// Nova sequência.
    pub const fn new(seed: u32) -> Self {
        Self {
            state: seed,
            frozen: false,
        }
    }

    /// Gerador sem perturbação (traço técnico).
    pub const fn frozen() -> Self {
        Self {
            state: 0,
            frozen: true,
        }
    }

    /// Próximo inteiro.
    pub fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_add(0x6d2b_79f5);
        let mut t = self.state;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 0x3d));
        t ^ (t >> 14)
    }

    /// Número em `[-1, 1)`.
    pub fn signed(&mut self) -> f32 {
        let raw = self.next_u32() >> 16;
        let unit = f32::from(u16::try_from(raw).unwrap_or(0)) / 65_535.0;
        unit * 2.0 - 1.0
    }

    fn offset(&mut self, amount: f32) -> f32 {
        if self.frozen {
            0.0
        } else {
            self.signed() * amount
        }
    }
}

/// Semente estável de um id (FNV-1a 32 bits).
pub fn seed_for(id: &str) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in id.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    hash
}

/// Executa o gerador no acabamento pedido: duas passagens no traço à mão, uma no exato.
pub(super) fn passes(ink: Ink, mut build: impl FnMut(&mut Rng) -> String) -> String {
    match ink {
        Ink::Rough(seed) => two_passes(seed, build),
        Ink::Exact => build(&mut Rng::frozen()),
    }
}

/// Executa duas passagens de um gerador e concatena os `d`.
pub(super) fn two_passes(seed: u32, mut build: impl FnMut(&mut Rng) -> String) -> String {
    let mut rng = Rng::new(seed);
    let first = build(&mut rng);
    let second = build(&mut rng);
    format!("{first} {second}")
}

pub(super) fn jitter(rng: &mut Rng, point: Point, amount: f32) -> Point {
    (point.0 + rng.offset(amount), point.1 + rng.offset(amount))
}
