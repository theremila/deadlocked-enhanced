//! bitset types backed by dynamic and fixed-size byte storage.

mod dynamic;
mod fixed;

use std::ops::Range;

pub use dynamic::DynamicBitSet;
pub use fixed::BitSet;

const BITS_PER_BYTE: usize = 8;

#[inline]
fn bytes_for_bits(bits: usize) -> usize {
    bits.div_ceil(BITS_PER_BYTE)
}

#[inline]
fn bitset_index(index: usize) -> (usize, usize) {
    (index / BITS_PER_BYTE, index % BITS_PER_BYTE)
}

#[inline]
fn bit_len(byte_len: usize) -> usize {
    byte_len * BITS_PER_BYTE
}

#[inline]
fn bitwise_not(bits: &[u8]) -> Vec<u8> {
    bits.iter().map(|byte| !byte).collect()
}

#[inline]
fn bitwise_binary_array<const BYTES: usize>(
    bits: &[u8; BYTES],
    other: &[u8; BYTES],
    op: impl Fn(u8, u8) -> u8,
) -> [u8; BYTES] {
    let mut out = [0; BYTES];

    for (index, (left, right)) in bits.iter().zip(other.iter()).enumerate() {
        out[index] = op(*left, *right);
    }

    out
}

#[inline]
fn bitwise_not_array<const BYTES: usize>(bits: &[u8; BYTES]) -> [u8; BYTES] {
    let mut out = [0; BYTES];

    for (index, byte) in bits.iter().enumerate() {
        out[index] = !byte;
    }

    out
}

#[inline]
fn get(bits: &[u8], index: usize) -> Option<bool> {
    if index >= bit_len(bits.len()) {
        return None;
    }

    let (byte, bit) = bitset_index(index);

    Some((bits[byte] >> bit) & 1 == 1)
}

#[inline]
fn set(bits: &mut [u8], index: usize, value: bool) {
    assert!(index < bit_len(bits.len()), "Index out of bounds");

    let (byte, bit) = bitset_index(index);

    if value {
        bits[byte] |= 1 << bit;
    } else {
        bits[byte] &= !(1 << bit);
    }
}

#[inline]
fn set_range(bits: &mut [u8], range: Range<usize>, value: bool) {
    if range.is_empty() {
        return;
    }

    assert!(range.end <= bit_len(bits.len()), "Index out of bounds");

    for index in range {
        set(bits, index, value);
    }
}

#[inline]
fn clear(bits: &mut [u8]) {
    bits.fill(0);
}

#[inline]
fn flip(bits: &mut [u8]) {
    for byte in bits {
        *byte = !*byte;
    }
}

#[inline]
fn count_ones(bits: &[u8]) -> usize {
    bits.iter().map(|byte| byte.count_ones() as usize).sum()
}

#[inline]
fn is_zeroed(bits: &[u8]) -> bool {
    bits.iter().all(|byte| *byte == 0)
}

#[inline]
fn iter(bits: &[u8]) -> impl Iterator<Item = bool> {
    bits.iter()
        .flat_map(|byte| (0..BITS_PER_BYTE).map(move |bit| (byte >> bit) & 1 == 1))
}

fn display_bitset(bits: &[u8], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "[")?;
    for bit in iter(bits) {
        write!(f, "{}", if bit { '1' } else { '0' })?;
    }
    write!(f, "]")
}
