use std::{io::Read, slice};
use thiserror::Error;

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

  fn read_bit(&mut self) -> Result<u8>;

  fn read_bits(&mut self, num: u8) -> Result<u64> {
    let mut result = 0u64;
    for _ in 0..num {
      result = result << 1 | self.read_bit()? as u64;
    }
    Ok(result)
  }

  fn read<T: BitField>(&mut self) -> Result<T>
  where
    Self: Sized,
  {
    T::read(self)
  }

  fn read_sized<T: SizedBitField>(&mut self, size: u8) -> Result<T>
  where
    Self: Sized,
  {
    T::read_sized(self, size)
  }
}

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

  fn read_bit(&mut self) -> Result<u8> {
    if self.pos == 8 {
      self
        .inner
        .read_exact(slice::from_mut(&mut self.buf))
        .or(Err(BitReaderError::NotEnoughData))?;
      self.pos = 0;
    }

    let value = (self.buf >> (7 - self.pos)) & 0b1;
    self.pos += 1;
    Ok(value)
  }
}

macro_rules! impl_read_sized_into_for_prim {
  ($ty:ty, $size:expr) => {
    impl SizedBitField for $ty {
      fn read_sized(reader: &mut impl BitRead, size: u8) -> Result<Self> {
        if size > $size {
          return Err(BitReaderError::TooLarge);
        }

        Ok(reader.read_bits(size)? as $ty)
      }
    }
  };
}

impl_read_sized_into_for_prim!(u8, 8);
impl_read_sized_into_for_prim!(i8, 8);
impl_read_sized_into_for_prim!(u16, 16);
impl_read_sized_into_for_prim!(i16, 16);
impl_read_sized_into_for_prim!(u32, 32);
impl_read_sized_into_for_prim!(i32, 32);
impl_read_sized_into_for_prim!(u64, 64);
impl_read_sized_into_for_prim!(i64, 64);
impl_read_sized_into_for_prim!(u128, 128);
impl_read_sized_into_for_prim!(i128, 128);

#[cfg(test)]
mod tests {
  use crate::*;
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
