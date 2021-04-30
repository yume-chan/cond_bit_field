use bit_stream::{BitField, BitStream, Result, cond_bit_field};
use serde::Serialize;

mod access_unit_delimiter;
mod exp_golomb;
mod header;
mod pic_param_set;
mod scaling_list;
mod seq_param_set;
mod slice;
mod slice_header;

pub use access_unit_delimiter::*;
pub use exp_golomb::*;
pub use header::*;
pub use pic_param_set::*;
pub use scaling_list::*;
pub use seq_param_set::*;
pub use slice::*;
pub use slice_header::*;

use crate::decoder::Decoder;

#[non_exhaustive]
#[derive(Clone, Debug, Serialize)]
pub enum NalUnitPayload {
    PictureParameterSet(PictureParameterSet),
    SequenceParameterSet(SequenceParameterSet),
    AccessUnitDelimiter(AccessUnitDelimiter),
    Unknown(Box<[u8]>),
}

impl<'a> BitField<'a> for NalUnitPayload {
    type Args = (&'a Decoder, &'a NalUnitHeader);

    fn read(stream: &mut BitStream, (decoder, header): Self::Args) -> Result<Self> {
        Ok(match header.ty {
            7 => stream.read(()).map(NalUnitPayload::SequenceParameterSet)?,
            8 => stream
                .read(decoder)
                .map(NalUnitPayload::PictureParameterSet)?,
            9 => stream.read(()).map(NalUnitPayload::AccessUnitDelimiter)?,
            _ => NalUnitPayload::Unknown(stream.read_all()),
        })
    }
}

cond_bit_field! {
    #[derive(Clone, Debug, Serialize)]
    #[extra_args(decoder: &Decoder)]
    pub struct NalUnit {
        pub header: NalUnitHeader;
        pub payload: NalUnitPayload[decoder, &header];
    }
}
