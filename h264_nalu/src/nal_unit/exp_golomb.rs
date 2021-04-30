use bit_stream::{BitField, BitStream, Result};
use derive_new_number::NewNumber;
use serde::Serialize;

/// § 9.1 Parsing process for Exp-Golomb codes
#[derive(Clone, Copy, Debug, Eq, Hash, NewNumber, PartialEq, Serialize)]
pub struct UnsignedExpGolombCode(pub u64);

impl<'a> BitField<'a> for UnsignedExpGolombCode {
    type Args = ();

    fn read(stream: &mut BitStream, _: ()) -> Result<Self> {
        let mut length = 0;
        while !stream.read_bit()? {
            length += 1;
        }

        if length == 0 {
            return Ok(Self(0));
        }

        Ok(Self((1 << length | stream.read::<u64>(length)?) - 1))
    }
}

/// § 9.1.1 Mapping process for signed Exp-Golomb codes
#[derive(Clone, Copy, Debug, Eq, Hash, NewNumber, PartialEq, Serialize)]
pub struct SignedExpGolombCode(pub i64);

impl<'a> BitField<'a> for SignedExpGolombCode {
    type Args = ();

    fn read(stream: &mut BitStream, _: ()) -> Result<Self> {
        let ue = stream.read::<UnsignedExpGolombCode>(())?.0;
        // Safety: `i64::MAX` equals to `u64::MAX >> 1`
        // So `(u64 >> 1) as i64` will never overflow
        Ok(Self(if ue % 1 == 1 {
            (ue >> 1 + 1) as i64
        } else {
            ((ue >> 1) as i64) * -1
        }))
    }
}
