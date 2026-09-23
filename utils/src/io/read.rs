use crate::io::{Endian, impl_read_endian_method, impl_read_method};
use std::io::{Read, Result};

/// extension methods for reading numbers and byte buffers from any reader.
pub trait ReadBytes: std::io::Read {
    impl_read_method!(read_u8, u8);

    impl_read_method!(read_u16, u16);
    impl_read_endian_method!(read_u16_endian, u16);

    impl_read_method!(read_u32, u32);
    impl_read_endian_method!(read_u32_endian, u32);

    impl_read_method!(read_u64, u64);
    impl_read_endian_method!(read_u64_endian, u64);

    impl_read_method!(read_u128, u128);
    impl_read_endian_method!(read_u128_endian, u128);

    impl_read_method!(read_usize, usize);
    impl_read_endian_method!(read_usize_endian, usize);

    impl_read_method!(read_i8, i8);

    impl_read_method!(read_i16, i16);
    impl_read_endian_method!(read_i16_endian, i16);

    impl_read_method!(read_i32, i32);
    impl_read_endian_method!(read_i32_endian, i32);

    impl_read_method!(read_i64, i64);
    impl_read_endian_method!(read_i64_endian, i64);

    impl_read_method!(read_i128, i128);
    impl_read_endian_method!(read_i128_endian, i128);

    impl_read_method!(read_isize, isize);
    impl_read_endian_method!(read_isize_endian, isize);

    impl_read_method!(read_f32, f32);
    impl_read_endian_method!(read_f32_endian, f32);

    impl_read_method!(read_f64, f64);
    impl_read_endian_method!(read_f64_endian, f64);

    /// reads exactly `count` bytes.
    #[inline]
    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; count];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    #[inline]
    fn read_array<const BYTES: usize>(&mut self) -> Result<[u8; BYTES]> {
        let mut buf = [0; BYTES];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// reads a null-terminated string.
    ///
    /// reads bytes until a null terminator (`\0`) is encountered,
    /// returning the accumulated bytes as a UTF-8 string.
    ///
    /// # Errors
    ///
    /// returns an error if the bytes are not valid UTF-8.
    #[inline]
    fn read_cstr(&mut self) -> Result<String> {
        let mut bytes = Vec::new();
        while let c = self.read_u8()?
            && c != 0
        {
            bytes.push(c);
        }
        String::from_utf8(bytes).map_err(|_| std::io::Error::other("Invalid UTF-8"))
    }
}

impl<R: std::io::Read> ReadBytes for R {}

pub struct EndianReader<R: Read> {
    inner: R,
    endian: Endian,
}

impl<R: Read> EndianReader<R> {
    pub fn new(inner: R, endian: Endian) -> Self {
        Self { inner, endian }
    }

    pub fn endian(&self) -> Endian {
        self.endian
    }
}

impl<R: Read> Read for EndianReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

macro_rules! impl_method {
    ($name:ident, $type:ty, $inner_fn:ident) => {
        #[inline]
        pub fn $name(&mut self) -> Result<$type> {
            self.inner.$inner_fn(self.endian)
        }
    };
}

impl<R: Read> EndianReader<R> {
    impl_method!(read_u16, u16, read_u16_endian);
    impl_method!(read_u32, u32, read_u32_endian);
    impl_method!(read_u64, u64, read_u64_endian);
    impl_method!(read_u128, u128, read_u128_endian);
    impl_method!(read_usize, usize, read_usize_endian);

    impl_method!(read_i16, i16, read_i16_endian);
    impl_method!(read_i32, i32, read_i32_endian);
    impl_method!(read_i64, i64, read_i64_endian);
    impl_method!(read_i128, i128, read_i128_endian);
    impl_method!(read_isize, isize, read_isize_endian);

    impl_method!(read_f32, f32, read_f32_endian);
    impl_method!(read_f64, f64, read_f64_endian);
}
