//! 07 §2 / §M.1: the source pins in the manifest are `sha256` values, so a build
//! verifies them. The workspace dependency allowlist (`check_deps`, 00 §4)
//! carries no hashing crate beyond `md-5`, and a pin check is not allowed to
//! grow a dependency, so this is the whole of SHA-256: FIPS 180-4 §4.2.2 in
//! plain integers, one call site, tested against the standard vectors.
//!
//! Scope is deliberate and narrow — **source pins only**. The state hash and the
//! genome's content seal stay `md-5` (07 §M.9, OD-49: identity, not security).

/// FIPS 180-4 §4.2.2.1: the first 32 bits of the fractional part of the cube
/// root of the first 64 primes.
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// FIPS 180-4 §5.3.3: the initial hash values.
const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

const fn rotate_right(value: u32, bits: u32) -> u32 {
    (value >> bits) | (value << (32 - bits))
}

/// `sha256(bytes)`, thirty-two bytes.
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut h = H0;

    let bit_len = (bytes.len() as u64) << 3;
    let mut padded = bytes.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for block in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (index, chunk) in block.chunks_exact(4).enumerate() {
            w[index] = u32::from_be_bytes(chunk.try_into().expect("4 bytes"));
        }
        for index in 16..64 {
            let s0 = rotate_right(w[index - 15], 7) ^ rotate_right(w[index - 15], 18) ^ (w[index - 15] >> 3);
            let s1 = rotate_right(w[index - 2], 17) ^ rotate_right(w[index - 2], 19) ^ (w[index - 2] >> 10);
            w[index] = w[index - 16]
                .wrapping_add(s0)
                .wrapping_add(w[index - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for index in 0..64 {
            let s1 = rotate_right(e, 6) ^ rotate_right(e, 11) ^ rotate_right(e, 25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[index])
                .wrapping_add(w[index]);
            let s0 = rotate_right(a, 2) ^ rotate_right(a, 13) ^ rotate_right(a, 22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        h = [
            h[0].wrapping_add(a),
            h[1].wrapping_add(b),
            h[2].wrapping_add(c),
            h[3].wrapping_add(d),
            h[4].wrapping_add(e),
            h[5].wrapping_add(f),
            h[6].wrapping_add(g),
            h[7].wrapping_add(hh),
        ];
    }

    let mut out = [0u8; 32];
    for (index, word) in h.iter().enumerate() {
        out[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// Lowercase hex, the form a manifest pin is written in.
pub fn hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex(&sha256(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_standard_vectors_hold() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(
            sha256_hex(&vec![b'a'; 1_000_000]),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    #[test]
    fn the_padding_branches_at_the_block_boundaries() {
        // 55 fits the length tail in the first block, 56 needs a second one, and
        // 64 is exactly one full block before padding.
        assert_eq!(
            sha256_hex(&vec![b'A'; 55]),
            "8963cc0afd622cc7574ac2011f93a3059b3d65548a77542a1559e3d202e6ab00"
        );
        assert_eq!(
            sha256_hex(&vec![b'A'; 56]),
            "6ea719cefa4b31862035a7fa606b7cc3602f46231117d135cc7119b3c1412314"
        );
        assert_eq!(
            sha256_hex(&vec![b'A'; 64]),
            "d53eda7a637c99cc7fb566d96e9fa109bf15c478410a3f5eb4d4c4e26cd081f6"
        );
    }

    #[test]
    fn the_constants_are_derived_not_trusted() {
        // A hand-typed table is the classic silent-corruption failure: one wrong
        // word and short messages still look plausible. Both tables are re-derived
        // here from their definitions — FIPS 180-4 §5.2.1 says "the first 32 bits
        // of the fractional part", and ⌊2³²·x⌋ & 0xffffffff *is* those bits.
        for (index, prime) in first_primes(64).iter().enumerate() {
            let scaled = floor_scaled_root(*prime, 3);
            assert_eq!(K[index], (scaled & 0xffff_ffff) as u32, "K[{index}]");
        }
        for (index, prime) in first_primes(8).iter().enumerate() {
            let scaled = floor_scaled_root(*prime, 2);
            assert_eq!(H0[index], (scaled & 0xffff_ffff) as u32, "H0[{index}]");
        }
    }

    fn first_primes(count: usize) -> Vec<u32> {
        let mut out = Vec::with_capacity(count);
        let mut candidate = 2u32;
        while out.len() < count {
            if is_prime(candidate) {
                out.push(candidate);
            }
            candidate += 1;
        }
        out
    }

    fn is_prime(value: u32) -> bool {
        if value < 2 {
            return false;
        }
        let mut divisor = 2u32;
        while divisor * divisor <= value {
            if value % divisor == 0 {
                return false;
            }
            divisor += 1;
        }
        true
    }

    /// ⌊2^32·ᵖ√value⌋, found by bisection on `xᵖ <= value << 32p`. Exact, and the
    /// primes here are small enough for `u128`.
    fn floor_scaled_root(value: u32, power: u32) -> u128 {
        let mut low = 0u128;
        let mut high = 1u128;
        while pow_le(high, power, value) {
            low = high;
            high *= 2;
        }
        let mut lo = low;
        let mut hi = high;
        while lo + 1 < hi {
            let mid = (lo + hi) / 2;
            if pow_le(mid, power, value) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        lo
    }

    fn pow_le(base: u128, power: u32, value: u32) -> bool {
        let mut acc = base;
        for _ in 1..power {
            acc = acc.saturating_mul(base);
        }
        acc <= (value as u128) << (32 * power)
    }
}
