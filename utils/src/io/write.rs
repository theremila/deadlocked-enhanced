use crate::io::{Endian, impl_write_endian_method, impl_write_method};
use std::{
    ffi::CString,
    io::{Result, Write},
};

/// extension methods for writing numbers and byte buffers to any writer.
pub trait WriteBytes: std::io::Write {
    impl_write_method!(write_u8, u8);

    impl_write_method!(write_u16, u16);
    impl_write_endian_method!(write_u16_endian, u16);

    impl_write_method!(write_u32, u32);
    impl_write_endian_method!(write_u32_endian, u32);

    impl_write_method!(write_u64, u64);
    impl_write_endian_method!(write_u64_endian, u64);

    impl_write_method!(write_u128, u128);
    impl_write_endian_method!(write_u128_endian, u128);

    impl_write_method!(write_usize, usize);
    impl_write_endian_method!(write_usize_endian, usize);

    impl_write_method!(write_i8, i8);

    impl_write_method!(write_i16, i16);
    impl_write_endian_method!(write_i16_endian, i16);

    impl_write_method!(write_i32, i32);
    impl_write_endian_method!(write_i32_endian, i32);

    impl_write_method!(write_i64, i64);
    impl_write_endian_method!(write_i64_endian, i64);

    impl_write_method!(write_i128, i128);
    impl_write_endian_method!(write_i128_endian, i128);

    impl_write_method!(write_isize, isize);
    impl_write_endian_method!(write_isize_endian, isize);

    impl_write_method!(write_f32, f32);
    impl_write_endian_method!(write_f32_endian, f32);

    impl_write_method!(write_f64, f64);
    impl_write_endian_method!(write_f64_endian, f64);

    /// writes exactly `bytes.len()` bytes.
    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.write_all(bytes)
    }

    /// writes a null-terminated string.
    ///
    /// converts `string` to a `CString` and writes all bytes including
    /// the null terminator to the underlying writer.
    ///
    /// # Errors
    ///
    /// returns an error if the string contains an embedded null byte.
    #[inline]
    fn write_cstr(&mut self, string: impl AsRef<str>) -> Result<()> {
        let cstr = CString::new(string.as_ref()).map_err(std::io::Error::other)?;
        let bytes = cstr.to_bytes_with_nul();
        self.write_all(bytes)
    }
}

impl<W: std::io::Write> WriteBytes for W {}

pub struct EndianWriter<W: Write> {
    inner: W,
    endian: Endian,
}

impl<W: Write> EndianWriter<W> {
    pub fn new(inner: W, endian: Endian) -> Self {
        Self { inner, endian }
    }

    pub fn endian(&self) -> Endian {
        self.endian
    }
}

impl<W: Write> Write for EndianWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

macro_rules! impl_method {
    ($name:ident, $type:ty, $inner_fn:ident) => {
        #[inline]
        pub fn $name(&mut self, value: $type) -> Result<()> {
            self.inner.$inner_fn(value, self.endian)
        }
    };
}

impl<W: Write> EndianWriter<W> {
    impl_method!(write_u16, u16, write_u16_endian);
    impl_method!(write_u32, u32, write_u32_endian);
    impl_method!(write_u64, u64, write_u64_endian);
    impl_method!(write_u128, u128, write_u128_endian);
    impl_method!(write_usize, usize, write_usize_endian);

    impl_method!(write_i16, i16, write_i16_endian);
    impl_method!(write_i32, i32, write_i32_endian);
    impl_method!(write_i64, i64, write_i64_endian);
    impl_method!(write_i128, i128, write_i128_endian);
    impl_method!(write_isize, isize, write_isize_endian);

    impl_method!(write_f32, f32, write_f32_endian);
    impl_method!(write_f64, f64, write_f64_endian);
}
