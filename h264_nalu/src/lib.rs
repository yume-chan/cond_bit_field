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

// 7.3.2.1.1 Sequence parameter set data syntax
cond_bit_field! {
  pub struct SequenceParameterSet {
    pub profile_idc: u8;

    pub constraint_set0_flag: bool;
    pub constraint_set1_flag: bool;
    pub constraint_set2_flag: bool;
    pub constraint_set3_flag: bool;
    pub constraint_set4_flag: bool;
    pub constraint_set5_flag: bool;
    _: 2;

    pub level_idc: u8;
    pub seq_parameter_set_id: ExpGolNumber;

    if profile_idc == 100 || profile_idc == 110 ||
      profile_idc == 122 || profile_idc == 244 || profile_idc == 44 ||
      profile_idc == 83 || profile_idc == 86 || profile_idc == 118 ||
      profile_idc == 128 || profile_idc == 138 || profile_idc == 139 ||
      profile_idc == 134 || profile_idc == 135 {
      pub chroma_format_idc: ExpGolNumber;

      if chroma_format_idc.0 == 3 {
        pub separate_colour_plane_flag: bool;
      }
    }
  }
}

// 7.3.2.2 Picture parameter set RBSP syntax
cond_bit_field! {
  pub struct PictureParameterSet {
    pub pic_parameter_set_id: ExpGolNumber;
    pub seq_parameter_set_id: ExpGolNumber;
    pub entropy_coding_mode_flag: bool;
    pub bottom_field_pic_order_in_frame_present_flag: bool;
    pub num_slice_groups_minus1: ExpGolNumber;

    if num_slice_groups_minus1.0 > 0 {
      pub slice_group_map_type: ExpGolNumber;

      if slice_group_map_type.0 == 0 {
        for _ in 0..num_slice_groups_minus1.0 {
          pub run_length_minus1: ExpGolNumber;
        }
      } else if slice_group_map_type.0 == 2 {
        for _ in 0..num_slice_groups_minus1.0 {
          pub top_left: ExpGolNumber;
          pub bottom_right: ExpGolNumber;
        }
      }
    }

    pub num_ref_idx_l0_default_active_minus1: ExpGolNumber;
    pub num_ref_idx_l1_default_active_minus1: ExpGolNumber;

    pub weighted_pred_flag: bool;
    pub weighted_bipred_idc: u2;
    pub pic_init_qp_minus26: ExpGolNumber;
    pub pic_init_qs_minus26: ExpGolNumber;
    pub chroma_qp_index_offset: ExpGolNumber;

    pub deblocking_filter_control_present_flag: bool;
    pub constrained_intra_pred_flag: bool;
    pub redundant_pic_cnt_present_flag: bool;
  }
}
