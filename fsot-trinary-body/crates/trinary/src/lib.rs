//! FSOT trinary primitives — **not binary**.
//!
//! Native state is a balanced ternary digit:
//!   SpinDown = −1  ·  Superposed = 0  ·  SpinUp = +1
//!
//! Continuous fields collapse into trits; packing to bits is only for
//! transport. Semantic computation stays in trit space.

#![cfg_attr(not(test), no_std)]
#![allow(dead_code)]

/// Collapse threshold for continuous → Trit (FSOT-aligned band around 0).
pub const COLLAPSE_THRESHOLD: f64 = 0.15;

/// Native word width: 27 trits (3³).
pub const TRIT_WORD_WIDTH: usize = 27;

/// Bytes needed to pack 27 trits (balanced ternary into ~42.7 bits → 6 bytes).
pub const WORD27_BYTES: usize = 6;

/// Balanced-ternary digit. Three genetic / logical states — never a bit.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(i8)]
pub enum Trit {
    /// Repress / damp / inhibit
    SpinDown = -1,
    /// Latent / undecided / open possibility
    Superposed = 0,
    /// Express / emerge / drive
    SpinUp = 1,
}

impl Trit {
    #[inline]
    pub fn to_i8(self) -> i8 {
        self as i8
    }

    #[inline]
    pub fn from_i8(v: i8) -> Trit {
        match v {
            i if i < 0 => Trit::SpinDown,
            0 => Trit::Superposed,
            _ => Trit::SpinUp,
        }
    }

    /// Biological short name for logging.
    pub fn name(self) -> &'static str {
        match self {
            Trit::SpinDown => "repress",
            Trit::Superposed => "latent",
            Trit::SpinUp => "express",
        }
    }
}

/// Continuous field → Trit via FSOT collapse band.
#[inline]
pub fn collapse(value: f64) -> Trit {
    if value > COLLAPSE_THRESHOLD {
        Trit::SpinUp
    } else if value < -COLLAPSE_THRESHOLD {
        Trit::SpinDown
    } else {
        Trit::Superposed
    }
}

/// Soft collapse: scale magnitude, keep superposed near zero.
#[inline]
pub fn collapse_soft(value: f64, threshold: f64) -> Trit {
    let t = if threshold > 0.0 {
        threshold
    } else {
        COLLAPSE_THRESHOLD
    };
    if value > t {
        Trit::SpinUp
    } else if value < -t {
        Trit::SpinDown
    } else {
        Trit::Superposed
    }
}

// ---- Single-trit gates ----------------------------------------------------

#[inline]
pub fn trit_not(a: Trit) -> Trit {
    match a {
        Trit::SpinDown => Trit::SpinUp,
        Trit::Superposed => Trit::Superposed,
        Trit::SpinUp => Trit::SpinDown,
    }
}

#[inline]
pub fn trit_and(a: Trit, b: Trit) -> Trit {
    Trit::from_i8(core::cmp::min(a.to_i8(), b.to_i8()))
}

#[inline]
pub fn trit_or(a: Trit, b: Trit) -> Trit {
    Trit::from_i8(core::cmp::max(a.to_i8(), b.to_i8()))
}

#[inline]
pub fn trit_mul(a: Trit, b: Trit) -> Trit {
    Trit::from_i8(a.to_i8() * b.to_i8())
}

/// Majority / consensus of three trits (genetic voting).
#[inline]
pub fn trit_consensus(a: Trit, b: Trit, c: Trit) -> Trit {
    let s = a.to_i8() as i32 + b.to_i8() as i32 + c.to_i8() as i32;
    if s >= 2 {
        Trit::SpinUp
    } else if s <= -2 {
        Trit::SpinDown
    } else {
        Trit::Superposed
    }
}

/// Trit sum with wrap (balanced ternary digit add, no carry out).
#[inline]
pub fn trit_sum(a: Trit, b: Trit) -> Trit {
    let s = a.to_i8() as i32 + b.to_i8() as i32;
    if s >= 2 {
        Trit::SpinDown
    } else if s <= -2 {
        Trit::SpinUp
    } else {
        Trit::from_i8(s as i8)
    }
}

// ---- Word27 ---------------------------------------------------------------

/// 27-trit machine word. Semantic layer of the body.
#[derive(Copy, Clone, Debug)]
pub struct Word27 {
    pub trits: [Trit; TRIT_WORD_WIDTH],
    pub width: u8,
}

impl Word27 {
    pub fn zero(width: u8) -> Self {
        let w = width.clamp(1, TRIT_WORD_WIDTH as u8);
        Word27 {
            trits: [Trit::Superposed; TRIT_WORD_WIDTH],
            width: w,
        }
    }

    /// Balanced-ternary encoding of a signed integer.
    pub fn encode(mut value: i64, width: u8) -> Self {
        let w = width.clamp(1, TRIT_WORD_WIDTH as u8);
        let mut out = Word27::zero(w);
        for i in 0..w as usize {
            let mut rem = value.rem_euclid(3);
            value = value.div_euclid(3);
            if rem == 2 {
                rem = -1;
                value += 1;
            }
            out.trits[i] = match rem {
                -1 => Trit::SpinDown,
                0 => Trit::Superposed,
                _ => Trit::SpinUp,
            };
        }
        out
    }

    pub fn decode(&self) -> i64 {
        let mut result: i64 = 0;
        let mut power: i64 = 1;
        for i in 0..self.width as usize {
            result += self.trits[i].to_i8() as i64 * power;
            power = power.saturating_mul(3);
        }
        result
    }

    pub fn word_not(&self) -> Self {
        let mut r = Word27::zero(self.width);
        for i in 0..self.width as usize {
            r.trits[i] = trit_not(self.trits[i]);
        }
        r
    }

    pub fn word_and(&self, other: &Self) -> Self {
        let w = self.width.min(other.width);
        let mut r = Word27::zero(w);
        for i in 0..w as usize {
            r.trits[i] = trit_and(self.trits[i], other.trits[i]);
        }
        r
    }

    pub fn word_or(&self, other: &Self) -> Self {
        let w = self.width.min(other.width);
        let mut r = Word27::zero(w);
        for i in 0..w as usize {
            r.trits[i] = trit_or(self.trits[i], other.trits[i]);
        }
        r
    }

    /// Count of each state — useful for genetic fabric summaries.
    pub fn tally(&self) -> (usize, usize, usize) {
        let mut down = 0usize;
        let mut mid = 0usize;
        let mut up = 0usize;
        for i in 0..self.width as usize {
            match self.trits[i] {
                Trit::SpinDown => down += 1,
                Trit::Superposed => mid += 1,
                Trit::SpinUp => up += 1,
            }
        }
        (down, mid, up)
    }

    /// Mean continuous field if each trit maps to its i8.
    pub fn mean_field(&self) -> f64 {
        if self.width == 0 {
            return 0.0;
        }
        let mut s = 0.0;
        for i in 0..self.width as usize {
            s += self.trits[i].to_i8() as f64;
        }
        s / self.width as f64
    }
}

// ---- Wire packing (transport only — not semantic storage) -----------------

/// Pack up to 27 trits into 6 bytes (balanced ternary → integer → LE bytes).
/// This exists for *wire/disk*, not as the body's native type.
pub fn pack6(w: &Word27) -> [u8; WORD27_BYTES] {
    let v = w.decode();
    let mut out = [0u8; WORD27_BYTES];
    let bytes = v.to_le_bytes();
    // i64 is 8 bytes; we keep low 6 (enough for 27-trit range ≈ ±3^27/2)
    out.copy_from_slice(&bytes[..WORD27_BYTES]);
    out
}

pub fn unpack6(bytes: &[u8; WORD27_BYTES]) -> Word27 {
    let mut full = [0u8; 8];
    full[..WORD27_BYTES].copy_from_slice(bytes);
    // sign-extend from 48-bit style: if bit 47 set, treat high as FF
    if full[5] & 0x80 != 0 {
        full[6] = 0xFF;
        full[7] = 0xFF;
    }
    let v = i64::from_le_bytes(full);
    Word27::encode(v, TRIT_WORD_WIDTH as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_states_not_two() {
        assert_ne!(Trit::SpinDown, Trit::Superposed);
        assert_ne!(Trit::Superposed, Trit::SpinUp);
        assert_ne!(Trit::SpinDown, Trit::SpinUp);
        assert_eq!(Trit::from_i8(-1), Trit::SpinDown);
        assert_eq!(Trit::from_i8(0), Trit::Superposed);
        assert_eq!(Trit::from_i8(1), Trit::SpinUp);
    }

    #[test]
    fn collapse_band() {
        assert_eq!(collapse(0.0), Trit::Superposed);
        assert_eq!(collapse(0.1), Trit::Superposed);
        assert_eq!(collapse(0.2), Trit::SpinUp);
        assert_eq!(collapse(-0.2), Trit::SpinDown);
    }

    #[test]
    fn encode_decode_roundtrip() {
        for v in [-1000i64, -1, 0, 1, 42, 12345] {
            let w = Word27::encode(v, 27);
            assert_eq!(w.decode(), v, "roundtrip {}", v);
        }
    }

    #[test]
    fn consensus_majority() {
        // Decisive when sum >= 2 or <= -2.
        assert_eq!(
            trit_consensus(Trit::SpinUp, Trit::SpinUp, Trit::SpinUp),
            Trit::SpinUp
        );
        assert_eq!(
            trit_consensus(Trit::SpinUp, Trit::SpinUp, Trit::Superposed),
            Trit::SpinUp
        ); // +1+1+0 = 2
        assert_eq!(
            trit_consensus(Trit::SpinDown, Trit::SpinDown, Trit::Superposed),
            Trit::SpinDown
        ); // -1-1+0 = -2
        assert_eq!(
            trit_consensus(Trit::SpinDown, Trit::SpinDown, Trit::SpinDown),
            Trit::SpinDown
        );
        // Non-decisive sums stay Superposed
        assert_eq!(
            trit_consensus(Trit::SpinUp, Trit::SpinDown, Trit::Superposed),
            Trit::Superposed
        ); // +1-1+0 = 0
        assert_eq!(
            trit_consensus(Trit::SpinUp, Trit::SpinUp, Trit::SpinDown),
            Trit::Superposed
        ); // +1+1-1 = +1
        assert_eq!(
            trit_consensus(Trit::SpinDown, Trit::SpinDown, Trit::SpinUp),
            Trit::Superposed
        ); // -1-1+1 = -1
    }

    #[test]
    fn pack_unpack_smoke() {
        let w = Word27::encode(99, 27);
        let b = pack6(&w);
        let w2 = unpack6(&b);
        assert_eq!(w2.decode(), 99);
    }
}
