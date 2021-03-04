use cond_bit_stream::{BitField, BitRead, Result};
use num_derive::{FromPrimitive, NumOps, ToPrimitive};

#[derive(FromPrimitive, NumOps, ToPrimitive, Debug)]
pub struct UnsignedExpGolombCode(pub u64);

// 7.2 Specification of syntax functions, categories, and descriptors
// 9.1 Parsing process for Exp-Golomb codes
impl BitField for UnsignedExpGolombCode {
    fn read(reader: &mut impl BitRead) -> Result<Self> {
        let mut length = 0;
        while !reader.read_bit()? {
            length += 1;
        }

        if length == 0 {
            return Ok(UnsignedExpGolombCode(0));
        }

        Ok(UnsignedExpGolombCode(
            1 << length | reader.read_sized::<u64, u8>(length)? - 1,
        ))
    }
}

#[derive(FromPrimitive, NumOps, ToPrimitive, Debug)]
pub struct SignedExpGolombCode(pub i64);

impl BitField for SignedExpGolombCode {
    fn read(reader: &mut impl BitRead) -> Result<Self> {
        let ue = reader.read::<UnsignedExpGolombCode>()?.0;
        Ok(Self(if ue % 1 == 1 {
            (ue >> 1 + 1) as i64
        } else {
            ((ue >> 1) as i64) * -1
        }))
    }
}
