use crate::{ScalingList, SignedExpGolombCode, UnsignedExpGolombCode};
use cond_bit_stream::cond_bit_field;

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
    pub seq_parameter_set_id: UnsignedExpGolombCode;

    if profile_idc == 100 || profile_idc == 110 ||
      profile_idc == 122 || profile_idc == 244 || profile_idc == 44 ||
      profile_idc == 83 || profile_idc == 86 || profile_idc == 118 ||
      profile_idc == 128 || profile_idc == 138 || profile_idc == 139 ||
      profile_idc == 134 || profile_idc == 135 {
      pub chroma_format_idc: UnsignedExpGolombCode;

      if chroma_format_idc.0 == 3 {
        pub separate_colour_plane_flag: bool;
      }

      pub bit_depth_luma_minus8: UnsignedExpGolombCode;
      pub bit_depth_chroma_minus8: UnsignedExpGolombCode;
      pub qpprime_y_zero_transform_bypass_flag: bool;
      pub seq_scaling_matrix_present_flag: bool;

      if seq_scaling_matrix_present_flag {
        for _ in 0..6 {
          #[size(16)]
          pub scaling_list_4x4: ScalingList;
        }

        for _ in 0..(if chroma_format_idc.0 != 3 { 2 } else { 6 }){
          #[size(64)]
          pub scaling_list_8x8: ScalingList;
        }
      }
    }

    pub log2_max_frame_num_minus4: UnsignedExpGolombCode;
    pub pic_order_cnt_type: UnsignedExpGolombCode;
    match pic_order_cnt_type.0 {
      0 => pub log2_max_pic_order_cnt_lsb_minus4: UnsignedExpGolombCode;
      1 => {
        pub delta_pic_order_always_zero_flag: bool;
        pub offset_for_non_ref_pic: SignedExpGolombCode;
        pub offset_for_top_to_bottom_field: SignedExpGolombCode;
        pub num_ref_frames_in_pic_order_cnt_cycle: UnsignedExpGolombCode;
        for _ in 0..num_ref_frames_in_pic_order_cnt_cycle.0 {
          pub offset_for_ref_frame: SignedExpGolombCode;
        }
      },
      _ => {}
    }

    pub max_num_ref_frames: UnsignedExpGolombCode;
    pub gaps_in_frame_num_value_allowed_flag: bool;
    pub pic_width_in_mbs_minus1: UnsignedExpGolombCode;
    pub pic_height_in_map_units_minus1: UnsignedExpGolombCode;
    pub frame_mbs_only_flag: bool;
    if !frame_mbs_only_flag {
      pub mb_adaptive_frame_field_flag: bool;
    }

    pub direct_8x8_inference_flag: bool;
    pub frame_cropping_flag: bool;
    if frame_cropping_flag {
      pub frame_crop_left_offset: UnsignedExpGolombCode;
      pub frame_crop_right_offset: UnsignedExpGolombCode;
      pub frame_crop_top_offset: UnsignedExpGolombCode;
      pub frame_crop_bottom_offset: UnsignedExpGolombCode;
    }

    pub vui_parameters_present_flag: bool;
    if vui_parameters_present_flag {
      // TODO
    }
  }
}
