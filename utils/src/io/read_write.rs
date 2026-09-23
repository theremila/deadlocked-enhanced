use crate::io::{Endian, ReadBytes, WriteBytes};
use std::io::{Read, Result, Write};

macro_rules! read_method {
    ($name:ident, $type:ty, $inner_name:ident) => {
        #[inline]
        pub fn $name(&mut self) -> Result<$type> {
            self.inner.$inner_name(self.endian)
        }
    };
}

macro_rules! write_method {
    ($name:ident, $type:ty, $inner_name:ident) => {
        #[inline]
        pub fn $name(&mut self, value: $type) -> Result<()> {
            self.inner.$inner_name(value, self.endian)
        }
    };
}

pub struct EndianReaderWriter<T: Read + Write> {
    inner: T,
    endian: Endian,
}

impl<T: Read + Write> EndianReaderWriter<T> {
    pub fn new(inner: T, endian: Endian) -> Self {
        Self { inner, endian }
    }

    pub fn endian(&self) -> Endian {
        self.endian
    }

    read_method!(read_u16, u16, read_u16_endian);
    read_method!(read_u32, u32, read_u32_endian);
    read_method!(read_u64, u64, read_u64_endian);
    read_method!(read_u128, u128, read_u128_endian);
    read_method!(read_usize, usize, read_usize_endian);

    read_method!(read_i16, i16, read_i16_endian);
    read_method!(read_i32, i32, read_i32_endian);
    read_method!(read_i64, i64, read_i64_endian);
    read_method!(read_i128, i128, read_i128_endian);
    read_method!(read_isize, isize, read_isize_endian);

    read_method!(read_f32, f32, read_f32_endian);
    read_method!(read_f64, f64, read_f64_endian);

    write_method!(write_u16, u16, write_u16_endian);
    write_method!(write_u32, u32, write_u32_endian);
    write_method!(write_u64, u64, write_u64_endian);
    write_method!(write_u128, u128, write_u128_endian);
    write_method!(write_usize, usize, write_usize_endian);

    write_method!(write_i16, i16, write_i16_endian);
    write_method!(write_i32, i32, write_i32_endian);
    write_method!(write_i64, i64, write_i64_endian);
    write_method!(write_i128, i128, write_i128_endian);
    write_method!(write_isize, isize, write_isize_endian);

    write_method!(write_f32, f32, write_f32_endian);
    write_method!(write_f64, f64, write_f64_endian);
}

impl<T: Read + Write> Read for EndianReaderWriter<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Read + Write> Write for EndianReaderWriter<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}
