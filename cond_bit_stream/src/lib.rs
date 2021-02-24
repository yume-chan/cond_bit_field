use std::io::Read;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BitReaderError {
  #[error("Not enough data")]
  NotEnoughData,
  #[error("Request size too large for current type")]
  TooLarge,
}

pub struct BitReader<R> {
  inner: R,
  buf: [u8; 1],
  pos: u8,
}

pub type Result<T> = std::result::Result<T, BitReaderError>;

impl<R: Read> BitReader<R> {
  pub fn new(inner: R) -> BitReader<R> {
    BitReader {
      inner,
      buf: [0],
      pos: 8,
    }
  }

  pub fn read_bit(&mut self) -> Result<u8> {
    if self.pos == 8 {
      self
        .inner
        .read_exact(&mut self.buf)
        .or(Err(BitReaderError::NotEnoughData))?;
      self.pos = 0;
    }

    Ok((self.buf[0] >> (7 - self.pos)) & 0b1)
  }

  pub fn read_bits(&mut self, num: u8) -> Result<u64> {
    let mut result = 0u64;
    for _ in 0..num {
      result = result << 1 | self.read_bit()? as u64;
    }
    Ok(result)
  }

  pub fn read<T: ReadInto>(&mut self) -> Result<T> {
    T::read(self)
  }

  pub fn read_sized<T: ReadSizedInto>(&mut self, size: u8) -> Result<T> {
    T::read_sized(self, size)
  }
}

pub trait ReadInto {
  fn read<R: Read>(reader: &mut BitReader<R>) -> Result<Self>
  where
    Self: Sized;
}

pub trait ReadSizedInto {
  fn read_sized<R: Read>(reader: &mut BitReader<R>, size: u8) -> Result<Self>
  where
    Self: Sized;
}

macro_rules! impl_read_sized_into_for_prim {
  ($ty:ty, $size:expr) => {
    impl ReadSizedInto for $ty {
      fn read_sized<R: Read>(reader: &mut BitReader<R>, size: u8) -> Result<Self> {
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
  use crate::BitReader;

  #[test]
  fn it_works() {
    let data = vec![0, 1, 2];
    let mut reader = BitReader::new(&data[..]);
    let a: u8 = reader.read_sized(8).unwrap();
    assert_eq!(a, 0);
  }
}
