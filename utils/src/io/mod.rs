//! helpers for reading and writing primitive values and raw bytes.

mod read;
mod read_write;
mod write;

pub use crate::io::{
    read::{EndianReader, ReadBytes},
    read_write::EndianReaderWriter,
    write::{EndianWriter, WriteBytes},
};

/// byte order used by endian-aware read and write helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    /// uses the platform native byte order.
    Native,
    /// uses little-endian byte order.
    Little,
    /// uses big-endian byte order.
    Big,
}

macro_rules! impl_read_method {
    ($name:ident, $type:ty) => {
        #[inline]
        fn $name(&mut self) -> Result<$type> {
            let mut buf = [0; size_of::<$type>()];
            self.read_exact(&mut buf)?;
            Ok(<$type>::from_ne_bytes(buf))
        }
    };
}

macro_rules! impl_read_endian_method {
    ($name:ident, $type:ty) => {
        #[inline]
        fn $name(&mut self, endian: Endian) -> Result<$type> {
            let mut buf = [0; size_of::<$type>()];
            self.read_exact(&mut buf)?;
            Ok(match endian {
                Endian::Native => <$type>::from_ne_bytes(buf),
                Endian::Little => <$type>::from_le_bytes(buf),
                Endian::Big => <$type>::from_be_bytes(buf),
            })
        }
    };
}

macro_rules! impl_write_method {
    ($name:ident, $type:ty) => {
        #[inline]
        fn $name(&mut self, value: $type) -> Result<()> {
            self.write_all(&value.to_ne_bytes())
        }
    };
}

macro_rules! impl_write_endian_method {
    ($name:ident, $type:ty) => {
        #[inline]
        fn $name(&mut self, value: $type, endian: Endian) -> Result<()> {
            let bytes = match endian {
                Endian::Native => value.to_ne_bytes(),
                Endian::Little => value.to_le_bytes(),
                Endian::Big => value.to_be_bytes(),
            };
            self.write_all(&bytes)
        }
    };
}

pub(crate) use {
    impl_read_endian_method, impl_read_method, impl_write_endian_method, impl_write_method,
};

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::{Endian, read::ReadBytes, write::WriteBytes};

    #[test]
    fn read_write_native_roundtrip() {
        let mut cursor = Cursor::new(Vec::new());

        cursor.write_u32(0x1234_5678).unwrap();
        cursor.write_i16(-1234).unwrap();
        cursor.write_f32(3.5).unwrap();

        cursor.set_position(0);

        assert_eq!(cursor.read_u32().unwrap(), 0x1234_5678);
        assert_eq!(cursor.read_i16().unwrap(), -1234);
        assert_eq!(cursor.read_f32().unwrap(), 3.5);
    }

    #[test]
    fn read_write_endian_roundtrip() {
        let mut cursor = Cursor::new(Vec::new());

        cursor.write_u32_endian(0x0102_0304, Endian::Big).unwrap();
        cursor.write_u16_endian(0x0506, Endian::Little).unwrap();

        cursor.set_position(0);

        assert_eq!(cursor.read_u32_endian(Endian::Big).unwrap(), 0x0102_0304);
        assert_eq!(cursor.read_u16_endian(Endian::Little).unwrap(), 0x0506);
    }

    #[test]
    fn endian_reader_writer_roundtrip() {
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut io = super::EndianReaderWriter::new(&mut cursor, Endian::Big);
            io.write_u32(0x0102_0304).unwrap();
            io.write_f32(3.5).unwrap();
        }

        cursor.set_position(0);
        let mut io = super::EndianReaderWriter::new(&mut cursor, Endian::Big);
        assert_eq!(io.read_u32().unwrap(), 0x0102_0304);
        assert_eq!(io.read_f32().unwrap(), 3.5);
    }

    #[test]
    fn read_write_bytes_roundtrip() {
        let mut cursor = Cursor::new(Vec::new());

        cursor.write_bytes(&[1, 2, 3, 4]).unwrap();
        cursor.set_position(0);

        assert_eq!(cursor.read_bytes(4).unwrap(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn read_write_cstr_roundtrip() {
        let mut cursor = Cursor::new(Vec::new());

        cursor.write_cstr("hello").unwrap();
        cursor.write_cstr("world").unwrap();

        cursor.set_position(0);

        assert_eq!(cursor.read_cstr().unwrap(), "hello");
        assert_eq!(cursor.read_cstr().unwrap(), "world");
    }

    #[test]
    fn read_write_cstr_empty_string() {
        let mut cursor = Cursor::new(Vec::new());

        cursor.write_bytes(&[0]).unwrap();
        cursor.set_position(0);

        assert_eq!(cursor.read_cstr().unwrap(), "");
    }

    #[test]
    fn read_write_cstr_with_special_chars() {
        let mut cursor = Cursor::new(Vec::new());

        cursor.write_cstr("hello world!").unwrap();
        cursor.set_position(0);

        assert_eq!(cursor.read_cstr().unwrap(), "hello world!");
    }

    #[test]
    fn read_cstr_invalid_utf8() {
        let mut cursor = Cursor::new(vec![0xff, 0xfe, 0x00]);

        let result = cursor.read_cstr();
        assert!(result.is_err());
    }
}
