use std::mem::size_of;

use thiserror::Error;

/// The error type for `BitStream`'s read operations
#[derive(Error, Debug)]
pub enum BitStreamError {
    /// The `BitStream` has not enough bits for the requested size.
    #[error("Not enough data")]
    NotEnoughData,

    /// The requested size doesn't fit into the result type.
    #[error("Requested size too large for result type")]
    TooLarge,
}

pub type Result<T> = ::std::result::Result<T, BitStreamError>;

/// The `BitField` trait defines how to `read` from a `BitStream`
pub trait BitField<'a>: Sized {
    /// Type of extra arguments for `read`.
    ///
    /// The exact meaning of each argument is defined by each type.
    /// For example, it can be the size of the value, or some other
    /// required data for reading the value.
    ///
    /// Can be `()` if no extra arguments are required,
    /// or a tuple type if multiple arguments are required.
    type Args;

    /// Reads from a `BitStream`.
    fn read(stream: &mut BitStream, args: Self::Args) -> Result<Self>;
}

/// A stream that can be read bit by bit
pub struct BitStream<'a> {
    data: &'a [u8],
    offset: usize,
    byte: u8,
    pos: u8,
}

impl<'a> BitStream<'a> {
    /// Creates a new `BitStream`.
    pub fn new(slice: &'a [u8]) -> Self {
        let buf = if slice.len() != 0 { slice[0] } else { 0 };
        Self {
            data: slice,
            offset: 0,
            byte: buf,
            pos: 0,
        }
    }

    /// Returns whether the `BitStream` is currently byte aligned
    pub fn byte_aligned(&self) -> bool {
        self.pos == 0 || self.pos == 8
    }

    /// Returns the remaining bit count in this stream.
    pub fn remaining(&self) -> usize {
        (self.data.len() - self.offset) * 8 - self.pos as usize
    }

    /// Skip (throw away) `bit_count` bits.
    pub fn skip(&mut self, bit_count: usize) -> Result<()> {
        let pos_overflow = self.pos as usize + bit_count;

        self.offset += pos_overflow / 8;
        if self.offset >= self.data.len() {
            return Err(BitStreamError::NotEnoughData);
        }

        self.byte = self.data[self.offset];
        self.pos = (pos_overflow % 8) as u8;
        Ok(())
    }

    /// Reads the next bit.
    ///
    /// Returns `true` if the bit is `1`, `false` for `0`
    pub fn read_bit(&mut self) -> Result<bool> {
        if self.pos == 8 {
            self.offset += 1;
            if self.offset >= self.data.len() {
                return Err(BitStreamError::NotEnoughData);
            }
            self.byte = self.data[self.offset];
            self.pos = 0;
        }

        let value = (self.byte >> (7 - self.pos)) & 0b1;
        self.pos += 1;
        Ok(value == 1)
    }

    /// Reads a `BitField`.
    pub fn read<'b, T: BitField<'b>>(&mut self, args: T::Args) -> Result<T> {
        T::read(self, args)
    }

    /// Reads all remaining bytes
    ///
    /// The stream must be byte aligned when `read_all` was called.
    pub fn read_all(&mut self) -> Box<[u8]> {
        if self.pos == 8 {
            self.offset += 1;
        }

        if self.offset == self.data.len() {
            return Box::new([]);
        }

        let mut data = Vec::with_capacity(self.data.len() - self.offset);
        data.copy_from_slice(&self.data[self.offset..]);
        self.offset = self.data.len();
        self.pos = 0;
        data.into_boxed_slice()
    }
}

macro_rules! impl_bit_field_for_signed {
    ($ty: ty) => {
        impl<'a> BitField<'a> for $ty {
            type Args = u8;

            fn read(stream: &mut BitStream, size: u8) -> Result<Self> {
                if size as usize > size_of::<$ty>() * 8 {
                    return Err(BitStreamError::TooLarge);
                }

                // -1: all bits are `1`
                let mut result: Self = if stream.read_bit()? { -1 } else { 0 };
                for _ in 0..(size - 1) {
                    result = result << 1 | Self::from(stream.read_bit()?);
                }

                Ok(result)
            }
        }
    };
}

impl_bit_field_for_signed!(i8);
impl_bit_field_for_signed!(i16);
impl_bit_field_for_signed!(i32);
impl_bit_field_for_signed!(i64);
impl_bit_field_for_signed!(i128);
impl_bit_field_for_signed!(isize);

macro_rules! impl_bit_field_for_unsigned {
    ($ty: ty) => {
        impl<'a> BitField<'a> for $ty {
            type Args = u8;

            fn read(stream: &mut BitStream, size: u8) -> Result<Self> {
                if size as usize > size_of::<$ty>() * 8 {
                    return Err(BitStreamError::TooLarge);
                }

                let mut result: Self = 0;
                for _ in 0..size {
                    result = result << 1 | Self::from(stream.read_bit()?);
                }

                Ok(result)
            }
        }
    };
}

impl_bit_field_for_unsigned!(u8);
impl_bit_field_for_unsigned!(u16);
impl_bit_field_for_unsigned!(u32);
impl_bit_field_for_unsigned!(u64);
impl_bit_field_for_unsigned!(u128);
impl_bit_field_for_unsigned!(usize);

#[cfg(test)]
mod tests {
    use crate as bit_stream;
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
