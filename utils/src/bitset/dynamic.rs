use std::{
    fmt::Display,
    ops::{Not, Range},
};

use crate::bitset::{
    BITS_PER_BYTE, bit_len, bitwise_not, bytes_for_bits, clear, count_ones, display_bitset, flip,
    get, is_zeroed, iter, set, set_range,
};

/// dynamic `BitSet` type, with an underlying [`Vec<u8>`].
/// grows when bits are set beyond the current length.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DynamicBitSet(Vec<u8>);

impl DynamicBitSet {
    /// creates an empty bitset.
    pub fn new() -> Self {
        Self::default()
    }

    /// creates an empty bitset with capacity for at least `capacity_bits` bits.
    pub fn with_capacity(capacity_bits: usize) -> Self {
        Self(vec![0; bytes_for_bits(capacity_bits)])
    }

    /// wraps an existing byte buffer as a bitset.
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }

    /// returns the underlying bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// consumes the bitset and returns the underlying bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }

    /// returns the number of bytes currently stored by the bitset.
    pub fn byte_len(&self) -> usize {
        self.0.len()
    }

    /// returns the number of addressable bits currently stored by the bitset.
    pub fn bit_len(&self) -> usize {
        bit_len(self.byte_len())
    }

    /// returns the underlying byte capacity.
    pub fn byte_capacity(&self) -> usize {
        self.0.capacity()
    }

    /// returns the bit capacity implied by the underlying byte capacity.
    pub fn bit_capacity(&self) -> usize {
        bit_len(self.byte_capacity())
    }

    /// returns the bit value at `index`, or `None` if it is out of bounds.
    pub fn get(&self, index: usize) -> Option<bool> {
        get(&self.0, index)
    }

    /// sets the bit at `index`, growing the bitset as needed.
    pub fn set(&mut self, index: usize, value: bool) {
        if index >= self.bit_len() {
            self.0.resize(index / BITS_PER_BYTE + 1, 0);
        }

        set(&mut self.0, index, value);
    }

    /// sets every bit in `range`, growing the bitset as needed.
    pub fn set_range(&mut self, range: Range<usize>, value: bool) {
        if range.end > self.bit_len() {
            self.0.resize(bytes_for_bits(range.end), 0);
        }

        set_range(&mut self.0, range, value);
    }

    /// clears all bits while preserving the current length.
    pub fn clear(&mut self) {
        clear(&mut self.0);
    }

    /// flips all bits.
    pub fn flip(&mut self) {
        flip(&mut self.0);
    }

    /// counts the number of set bits.
    pub fn count_ones(&self) -> usize {
        count_ones(&self.0)
    }

    /// returns `true` if all stored bits are unset.
    pub fn is_zeroed(&self) -> bool {
        is_zeroed(&self.0)
    }

    /// iterates over every stored bit in least-significant-bit-first order.
    pub fn iter(&self) -> impl Iterator<Item = bool> {
        iter(&self.0)
    }
}

impl Display for DynamicBitSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_bitset(&self.0, f)
    }
}

impl AsRef<[u8]> for DynamicBitSet {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<Vec<u8>> for DynamicBitSet {
    fn from(bytes: Vec<u8>) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<&[u8]> for DynamicBitSet {
    fn from(bytes: &[u8]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl<const BYTES: usize> From<[u8; BYTES]> for DynamicBitSet {
    fn from(bytes: [u8; BYTES]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<DynamicBitSet> for Vec<u8> {
    fn from(bitset: DynamicBitSet) -> Self {
        bitset.into_bytes()
    }
}

impl Not for DynamicBitSet {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(bitwise_not(&self.0))
    }
}

impl Not for &DynamicBitSet {
    type Output = DynamicBitSet;

    fn not(self) -> Self::Output {
        DynamicBitSet(bitwise_not(&self.0))
    }
}

#[cfg(test)]
mod test {
    use super::DynamicBitSet;

    #[test]
    fn set_get_roundtrip() {
        let mut bitset = DynamicBitSet::new();

        bitset.set(0, true);
        bitset.set(7, true);
        bitset.set(8, true);
        bitset.set(15, false);

        assert_eq!(bitset.get(0), Some(true));
        assert_eq!(bitset.get(7), Some(true));
        assert_eq!(bitset.get(8), Some(true));
        assert_eq!(bitset.get(15), Some(false));
    }

    #[test]
    fn clear_and_is_zeroed() {
        let mut bitset = DynamicBitSet::new();

        assert!(bitset.is_zeroed());

        bitset.set(3, true);
        assert!(!bitset.is_zeroed());

        bitset.clear();
        assert!(bitset.is_zeroed());
        assert_eq!(bitset.bit_len(), 8);
    }

    #[test]
    fn iter_matches_bits() {
        let mut bitset = DynamicBitSet::new();

        bitset.set(0, true);
        bitset.set(3, true);
        bitset.set(8, true);

        let values: Vec<bool> = bitset.iter().collect();

        assert_eq!(values.len(), bitset.bit_len());
        assert!(values[0]);
        assert!(!values[1]);
        assert!(values[3]);
        assert!(values[8]);
    }

    #[test]
    fn set_range_and_count() {
        let mut bitset = DynamicBitSet::new();

        bitset.set_range(2..6, true);
        assert_eq!(bitset.count_ones(), 4);
        assert_eq!(bitset.get(2), Some(true));
        assert_eq!(bitset.get(5), Some(true));
        assert_eq!(bitset.get(6), Some(false));
    }

    #[test]
    fn capacity_is_reported_in_bits_and_bytes() {
        let bitset = DynamicBitSet::with_capacity(9);

        assert_eq!(bitset.byte_capacity(), 2);
        assert_eq!(bitset.bit_capacity(), 16);
        assert!(bitset.is_zeroed());
    }

    #[test]
    fn from_and_into_bytes_roundtrip() {
        let bitset = DynamicBitSet::from_bytes([0b0000_0101, 0b0000_0010]);

        assert_eq!(bitset.as_bytes(), &[0b0000_0101, 0b0000_0010]);
        assert_eq!(bitset.into_bytes(), vec![0b0000_0101, 0b0000_0010]);
    }

    #[test]
    fn trait_conversions() {
        let bitset = DynamicBitSet::from([0b0000_0101, 0b0000_0010]);

        assert_eq!(bitset.as_ref(), &[0b0000_0101, 0b0000_0010]);
        assert_eq!(Vec::<u8>::from(bitset), vec![0b0000_0101, 0b0000_0010]);
    }

    #[test]
    fn get_out_of_bounds_returns_none() {
        let bitset = DynamicBitSet::new();

        assert_eq!(bitset.get(0), None);
    }

    #[test]
    fn display() {
        let mut bitset = DynamicBitSet::new();

        bitset.set(2, true);
        bitset.set(4, true);

        assert_eq!(format!("{bitset}"), "[00101000]");
    }
}
