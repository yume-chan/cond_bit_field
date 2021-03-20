use bit_stream::{cond_bit_field, BitField, BitStream, Result};
use serde::Serialize;

mod access_unit_delimiter;
mod exp_golomb;
mod pic_param_set;
mod scaling_list;
mod seq_param_set;

pub use access_unit_delimiter::*;
pub use exp_golomb::*;
pub use pic_param_set::*;
pub use scaling_list::*;
pub use seq_param_set::*;

#[non_exhaustive]
#[derive(Clone, Debug, Serialize)]
pub enum NalUnitPayload {
    PictureParameterSet(PictureParameterSet),
    SequenceParameterSet(SequenceParameterSet),
    AccessUnitDelimiter(AccessUnitDelimiter),
    Unknown(Box<[u8]>),
}

cond_bit_field! {
    // 7.3.1 NAL unit syntax
    #[derive(Clone, Copy, Debug, Serialize)]
    pub struct NalUnitHeader {
        _: 1;

        pub ref_idc: u2;
        pub ty: u5;

        match ty {
            14 | 20 | 21 => {
                if ty != 21 {
                    pub svc_extension_flag: bool;
                } else{
                    pub avc_3d_extension_flag: bool;
                }

                if svc_extension_flag == Some(true) {
                    // TODO nal_unit_header_svc_extension
                } else if avc_3d_extension_flag == Some(true) {
                    // TODO nal_unit_header_3davc_extension
                } else {
                    // TODO nal_unit_header_mvc_extension
                }
            },
            _ => {}
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct NalUnit {
    pub header: NalUnitHeader,
    pub payload: NalUnitPayload,
}

impl BitField for NalUnit {
    fn read(stream: &mut BitStream) -> Result<Self> {
        let header = stream.read::<NalUnitHeader>()?;
        // Table 7-1 – NAL unit type codes, syntax element categories, and NAL unit type classes
        let payload = match header.ty {
            7 => NalUnitPayload::SequenceParameterSet(stream.read()?),
            8 => NalUnitPayload::PictureParameterSet(stream.read()?),
            9 => NalUnitPayload::AccessUnitDelimiter(stream.read()?),
            _ => NalUnitPayload::Unknown(stream.read_all()),
        };
        Ok(Self { header, payload })
    }
}
