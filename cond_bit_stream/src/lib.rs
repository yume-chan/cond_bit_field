use std::{convert::TryInto, io::Read, mem::size_of, slice};
use thiserror::Error;

pub use cond_bit_field::*;

#[derive(Error, Debug)]
pub enum BitReaderError {
  #[error("Not enough data")]
  NotEnoughData,
  #[error("Request size too large for current type")]
  TooLarge,
}

pub trait BitField {
  fn read(reader: &mut impl BitRead) -> Result<Self>
  where
    Self: Sized;
}

pub trait SizedBitField {
  fn read_sized(reader: &mut impl BitRead, size: u8) -> Result<Self>
  where
    Self: Sized;
}

pub trait BitRead {
  fn skip(&mut self, size: u8) -> Result<()>;

  fn read_bit(&mut self) -> Result<bool>;

  fn read<T: BitField>(&mut self) -> Result<T>
  where
    Self: Sized,
  {
    T::read(self)
  }

  fn read_sized<T: SizedBitField, S: TryInto<u8>>(&mut self, size: S) -> Result<T>
  where
    Self: Sized,
  {
    T::read_sized(self, size.try_into().or(Err(BitReaderError::TooLarge))?)
  }
}

macro_rules! impl_read_sized_for_signed {
  ($ty: ty) => {
    impl SizedBitField for $ty {
      fn read_sized(reader: &mut impl BitRead, size: u8) -> Result<Self> {
        let size: u8 = size.try_into().or(Err(BitReaderError::TooLarge))?;
        if size as usize > size_of::<$ty>() {
          return Err(BitReaderError::TooLarge);
        }

        let mut result: Self = if reader.read_bit()? { -1 } else { 0 };
        for _ in 0..(size - 1) {
          result = result << 1 | Self::from(reader.read_bit()?);
        }

        Ok(result)
      }
    }
  };
}

impl_read_sized_for_signed!(i8);
impl_read_sized_for_signed!(i16);
impl_read_sized_for_signed!(i32);
impl_read_sized_for_signed!(i64);

macro_rules! impl_read_sized_for_unsigned {
  ($ty: ty) => {
    impl SizedBitField for $ty {
      fn read_sized(reader: &mut impl BitRead, size: u8) -> Result<Self> {
        if size as usize > size_of::<$ty>() {
          return Err(BitReaderError::TooLarge);
        }

        let mut result: Self = 0;
        for _ in 0..size {
          result = result << 1 | Self::from(reader.read_bit()?);
        }

        Ok(result)
      }
    }
  };
}

impl_read_sized_for_unsigned!(u8);
impl_read_sized_for_unsigned!(u16);
impl_read_sized_for_unsigned!(u32);
impl_read_sized_for_unsigned!(u64);

pub struct BitReader<R> {
  inner: R,
  buf: u8,
  pos: u8,
}

pub type Result<T> = std::result::Result<T, BitReaderError>;

impl<R: Read> BitReader<R> {
  pub fn new(inner: R) -> BitReader<R> {
    BitReader {
      inner,
      buf: 0,
      pos: 8,
    }
  }
}

impl<R: Read> BitRead for BitReader<R> {
  fn skip(&mut self, mut size: u8) -> Result<()> {
    if self.pos + size > 7 {
      size -= 7 - self.pos;
    }

    let bytes = (size as f32 / 8f32).ceil() as usize;
    let mut buf = vec![0; bytes];
    self
      .inner
      .read_exact(&mut buf)
      .or(Err(BitReaderError::NotEnoughData))?;

    self.pos = size % 8;
    self.buf = buf[bytes - 1];
    Ok(())
  }

  fn read_bit(&mut self) -> Result<bool> {
    if self.pos == 8 {
      self
        .inner
        .read_exact(slice::from_mut(&mut self.buf))
        .or(Err(BitReaderError::NotEnoughData))?;
      self.pos = 0;
    }

    let value = (self.buf >> (7 - self.pos)) & 0b1;
    self.pos += 1;
    Ok(value == 1)
  }
}

#[cfg(test)]
mod tests {
  use crate as cond_bit_stream;
  use cond_bit_field::cond_bit_field;

  cond_bit_field! {
    struct Simple {
      pub a: bool;
      pub b: i1;
      _: 10;
      pub c: u1;
      pub d: i4;
      pub e: u4;
      _: 10;
      pub f: i7;
      pub g: u7;
      pub h: i14;
      pub i: u14;
      _: 10;
      pub j: i18;
      pub k: u18;
      _: 10;
    }
  }

  cond_bit_field! {
    struct IfWithoutElse {
      pub a: bool;
      _: 10;

      if a {
        _: 10;
        pub b: u3;
        _: 10;
        pub c: i15;
        _: 10;
      }

      _: 10;
    }
  }

  cond_bit_field! {
    struct IfElse {
      pub a: bool;
      _: 10;

      if a {
        _: 10;
        pub b: u3;
        _: 10;
        pub c: i15;
        _: 10;
      } else {
        _: 10;
        pub d: i4;
        _: 10;
        pub e: u10;
        _: 10;
      }

      _: 10;
      pub z: bool;
    }
  }

  cond_bit_field! {
    struct IfElseIf {
      pub a: bool;
      _: 10;

      if a {
        _: 10;
        pub b: u3;
        _: 10;
        pub c: i15;
        _: 10;
      } else if !a {
        _: 10;
        pub d: i4;
        _: 10;
        pub e: u10;
        _: 10;
      }

      _: 10;
      pub z: bool;
    }
  }

  cond_bit_field! {
    struct IfElseIfElse {
      pub a: bool;
      _: 10;

      if a {
        _: 10;
        pub b: u3;
        _: 10;
        pub c: i15;
        _: 10;
      } else if !a {
        _: 10;
        pub d: i4;
        _: 10;
        pub e: u10;
        _: 10;
      } else {
        pub f: bool;
        _: 10;
        pub g: i33;
        _: 10;
      }

      _: 10;
      pub z: bool;
    }
  }

  cond_bit_field! {
    struct IfElseIfElseIf {
      pub a: bool;
      pub b: bool;
      _: 10;

      if a {
        pub c: u3;
        _: 10;
        pub d: i15;
        _: 10;
      } else if !a {
        _: 10;
        pub e: i4;
        _: 10;
        pub f: u10;
      } else if b {
        pub g: bool;
        _: 10;
        pub h: i33;
        _: 10;
      }

      pub z: bool;
    }
  }

  cond_bit_field! {
    struct RecursiveIf {
      pub a: bool;
      pub b: bool;
      _: 10;

      if a {
        pub c: u3;
        _: 10;
        pub d: i15;
        _: 10;

        if !a {
          pub e: i4;
          _: 10;
          pub f: u10;
          _: 10;
        } else if b {
          pub g: bool;
          _: 10;
          pub h: i33;
          _: 10;
        }

        _: 10;
        pub z: bool;
      }
    }
  }

  cond_bit_field! {
    struct EmptyForLoop {
      if true {
        for _ in 0..100 {

        }
      }
    }
  }
}
