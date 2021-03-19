use cond_bit_stream::{cond_bit_field, BitField, BitRead, Result};
use serde::Serialize;

use crate::{PictureParameterSet, SequenceParameterSet};

#[non_exhaustive]
#[derive(Clone, Debug, Serialize)]
pub enum NalUnitPayload {
  PictureParameterSet(PictureParameterSet),
  SequenceParameterSet(SequenceParameterSet),
  Unknown(),
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
  fn read(reader: &mut impl BitRead) -> Result<Self> {
    let header = reader.read::<NalUnitHeader>()?;
    let payload = match header.ty {
      7 => NalUnitPayload::SequenceParameterSet(reader.read()?),
      8 => NalUnitPayload::PictureParameterSet(reader.read()?),
      _ => NalUnitPayload::Unknown(),
    };
    Ok(Self { header, payload })
  }
}
