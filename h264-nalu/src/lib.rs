use cond_bit_field::cond_bit_field;
use cond_bit_stream::{BitField, BitRead, Result};
use num_derive::{FromPrimitive, NumOps, ToPrimitive};

#[derive(FromPrimitive, NumOps, ToPrimitive)]
pub struct ExpGolNumber(u64);

impl BitField for ExpGolNumber {
  fn read(reader: &mut impl BitRead) -> Result<Self> {
    let mut length = 0;
    while reader.read_bit()? == 0 {
      length += 1;
    }

    if length == 0 {
      return Ok(ExpGolNumber(0));
    }

    Ok(ExpGolNumber(1 << length | reader.read_bits(length)? - 1))
  }
}

cond_bit_field! {
  pub struct PictureParameterSet {
    pub pic_parameter_set_id: ExpGolNumber;
    pub seq_parameter_set_id: ExpGolNumber;
    pub entropy_coding_mode_flag: bool;
    pub bottom_field_pic_order_in_frame_present_flag: bool;
    pub num_slice_groups_minus1: ExpGolNumber;

    if num_slice_groups_minus1.0 > 1 {
      pub slice_group_map_type: ExpGolNumber;

      if slice_group_map_type.0 == 0 {

      }
    }

    pub num_ref_idx_l0_default_active_minus1: ExpGolNumber;
    pub num_ref_idx_l1_default_active_minus1: ExpGolNumber;

    pub weighted_pred_flag: bool;
  }
}
