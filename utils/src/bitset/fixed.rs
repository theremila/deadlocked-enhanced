use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Range},
};

use crate::bitset::{
    BITS_PER_BYTE, bitwise_binary_array, bitwise_not_array, clear, count_ones, display_bitset,
    flip, get, is_zeroed, iter, set, set_range,
};

/// fixed `BitSet` type, with an underlying `[u8; BYTES]`.
/// will panic if trying to set indices out of bounds.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitSet<const BYTES: usize>([u8; BYTES]);

impl<const BYTES: usize> BitSet<BYTES> {
    /// creates a zero-initialized fixed-size bitset.
    pub fn new() -> Self {
        Self::default()
    }

    /// wraps an existing byte array as a fixed-size bitset.
    pub fn from_bytes(bytes: [u8; BYTES]) -> Self {
        Self(bytes)
    }

    /// returns the underlying bytes.
    pub fn as_bytes(&self) -> &[u8; BYTES] {
        &self.0
    }

    /// consumes the bitset and returns the underlying bytes.
    pub fn into_bytes(self) -> [u8; BYTES] {
        self.0
    }

    /// returns the fixed byte length.
    pub const fn byte_len(&self) -> usize {
        BYTES
    }

    /// returns the fixed bit length.
    pub const fn bit_len(&self) -> usize {
        BYTES * BITS_PER_BYTE
    }

    /// sets the bit at `index`.
    ///
    /// panics if `index` is out of bounds.
    pub fn set(&mut self, index: usize, value: bool) {
        set(&mut self.0, index, value);
    }

    /// sets every bit in `range`.
    ///
    /// panics if `range` exceeds the fixed size.
    pub fn set_range(&mut self, range: Range<usize>, value: bool) {
        set_range(&mut self.0, range, value);
    }

    /// returns the bit value at `index`, or `None` if it is out of bounds.
    pub fn get(&self, index: usize) -> Option<bool> {
        get(&self.0, index)
    }

    /// clears all bits.
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

    /// returns `true` if all bits are unset.
    pub fn is_zeroed(&self) -> bool {
        is_zeroed(&self.0)
    }

    /// iterates over every stored bit in least-significant-bit-first order.
    pub fn iter(&self) -> impl Iterator<Item = bool> {
        iter(&self.0)
    }
}

impl<const BYTES: usize> Default for BitSet<BYTES> {
    fn default() -> Self {
        Self([0; BYTES])
    }
}

impl<const BYTES: usize> Display for BitSet<BYTES> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_bitset(&self.0, f)
    }
}

impl<const BYTES: usize> AsRef<[u8]> for BitSet<BYTES> {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<const BYTES: usize> AsRef<[u8; BYTES]> for BitSet<BYTES> {
    fn as_ref(&self) -> &[u8; BYTES] {
        self.as_bytes()
    }
}

impl<const BYTES: usize> From<[u8; BYTES]> for BitSet<BYTES> {
    fn from(bytes: [u8; BYTES]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl<const BYTES: usize> From<BitSet<BYTES>> for [u8; BYTES] {
    fn from(bitset: BitSet<BYTES>) -> Self {
        bitset.into_bytes()
    }
}

macro_rules! impl_bitwise_op {
    ($op:ident, $op_assign:ident, $method:ident, $method_assign:ident, $op_fn:expr) => {
        impl<const BYTES: usize> $op for BitSet<BYTES> {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self::Output {
                Self(bitwise_binary_array(&self.0, &rhs.0, $op_fn))
            }
        }

        impl<const BYTES: usize> $op<&BitSet<BYTES>> for BitSet<BYTES> {
            type Output = Self;
            fn $method(self, rhs: &Self) -> Self::Output {
                Self(bitwise_binary_array(&self.0, &rhs.0, $op_fn))
            }
        }

        impl<const BYTES: usize> $op<BitSet<BYTES>> for &BitSet<BYTES> {
            type Output = BitSet<BYTES>;
            fn $method(self, rhs: BitSet<BYTES>) -> Self::Output {
                BitSet(bitwise_binary_array(&self.0, &rhs.0, $op_fn))
            }
        }

        impl<const BYTES: usize> $op for &BitSet<BYTES> {
            type Output = BitSet<BYTES>;
            fn $method(self, rhs: Self) -> Self::Output {
                BitSet(bitwise_binary_array(&self.0, &rhs.0, $op_fn))
            }
        }

        impl<const BYTES: usize> $op_assign<&Self> for BitSet<BYTES> {
            fn $method_assign(&mut self, rhs: &Self) {
                *self = Self(bitwise_binary_array(&self.0, &rhs.0, $op_fn));
            }
        }

        impl<const BYTES: usize> $op_assign for BitSet<BYTES> {
            fn $method_assign(&mut self, rhs: Self) {
                *self = Self(bitwise_binary_array(&self.0, &rhs.0, $op_fn));
            }
        }
    };
}

impl_bitwise_op!(
    BitAnd,
    BitAndAssign,
    bitand,
    bitand_assign,
    |left, right| left & right
);

impl_bitwise_op!(BitOr, BitOrAssign, bitor, bitor_assign, |left, right| left
    | right);

impl_bitwise_op!(
    BitXor,
    BitXorAssign,
    bitxor,
    bitxor_assign,
    |left, right| left ^ right
);

impl<const BYTES: usize> Not for BitSet<BYTES> {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(bitwise_not_array(&self.0))
    }
}

impl<const BYTES: usize> Not for &BitSet<BYTES> {
    type Output = BitSet<BYTES>;

    fn not(self) -> Self::Output {
        BitSet(bitwise_not_array(&self.0))
    }
}

#[cfg(test)]
mod test {
    use super::BitSet;

    #[test]
    fn set_get_roundtrip() {
        let mut bitset = BitSet::<4>::new();

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
        let mut bitset = BitSet::<4>::new();

        assert!(bitset.is_zeroed());

        bitset.set(3, true);
        assert!(!bitset.is_zeroed());

        bitset.clear();
        assert!(bitset.is_zeroed());
    }

    #[test]
    fn iter_matches_bits() {
        let mut bitset = BitSet::<4>::new();

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
        let mut bitset = BitSet::<4>::new();

        bitset.set_range(2..6, true);
        assert_eq!(bitset.count_ones(), 4);
        assert_eq!(bitset.get(2), Some(true));
        assert_eq!(bitset.get(5), Some(true));
        assert_eq!(bitset.get(6), Some(false));
    }

    #[test]
    fn display() {
        let mut bitset = BitSet::<1>::new();

        bitset.set(2, true);
        bitset.set(4, true);

        assert_eq!(format!("{bitset}"), "[00101000]");
    }

    #[test]
    fn from_and_into_bytes_roundtrip() {
        let bitset = BitSet::<2>::from_bytes([0b0000_0101, 0b0000_0010]);

        assert_eq!(bitset.as_bytes(), &[0b0000_0101, 0b0000_0010]);
        assert_eq!(bitset.into_bytes(), [0b0000_0101, 0b0000_0010]);
    }

    #[test]
    fn trait_conversions() {
        let bitset = BitSet::<2>::from([0b0000_0101, 0b0000_0010]);

        assert_eq!(AsRef::<[u8]>::as_ref(&bitset), &[0b0000_0101, 0b0000_0010]);
        assert_eq!(
            AsRef::<[u8; 2]>::as_ref(&bitset),
            &[0b0000_0101, 0b0000_0010]
        );
        assert_eq!(<[u8; 2]>::from(bitset), [0b0000_0101, 0b0000_0010]);
    }

    #[test]
    fn bitwise_operations() {
        let left = BitSet::<2>::from([0b0000_1100, 0b1010_1010]);
        let right = BitSet::<2>::from([0b0000_1010, 0b1100_1100]);

        assert_eq!((left & right).as_bytes(), &[0b0000_1000, 0b1000_1000]);
        assert_eq!((left | right).as_bytes(), &[0b0000_1110, 0b1110_1110]);
        assert_eq!((left ^ right).as_bytes(), &[0b0000_0110, 0b0110_0110]);
        assert_eq!((!&left).as_bytes(), &[0b1111_0011, 0b0101_0101]);
    }

    #[test]
    fn bitwise_assign_operations() {
        let original = BitSet::<2>::from([0b0000_1100, 0b1010_1010]);
        let other = BitSet::<2>::from([0b0000_1010, 0b1100_1100]);

        let mut anded = original;
        anded &= other;
        assert_eq!(anded.as_bytes(), &[0b0000_1000, 0b1000_1000]);

        let mut ored = original;
        ored |= other;
        assert_eq!(ored.as_bytes(), &[0b0000_1110, 0b1110_1110]);

        let mut xored = original;
        xored ^= other;
        assert_eq!(xored.as_bytes(), &[0b0000_0110, 0b0110_0110]);
    }
}
