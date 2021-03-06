use crate::{nal_unit::{ScalingList, SignedExpGolombCode, UnsignedExpGolombCode},
            Decoder};
use bit_stream::cond_bit_field;
use serde::Serialize;

cond_bit_field! {
    /// pic_parameter_set_rbsp
    ///
    /// § 7.3.2.2 Picture parameter set RBSP syntax
    #[derive(Clone, Debug, Serialize)]
    #[extra_args(decoder: &Decoder)]
    pub struct PictureParameterSet {
        pub pic_parameter_set_id: UnsignedExpGolombCode;
        pub seq_parameter_set_id: UnsignedExpGolombCode;
        pub entropy_coding_mode_flag: bool;
        pub bottom_field_pic_order_in_frame_present_flag: bool;
        pub num_slice_groups_minus1: UnsignedExpGolombCode;

        if num_slice_groups_minus1 > 0 {
            pub slice_group_map_type: UnsignedExpGolombCode;

            match slice_group_map_type.0 {
                0 => {
                    for _ in 0..=num_slice_groups_minus1.0 {
                        pub run_length_minus1: UnsignedExpGolombCode;
                    }
                }
                2 => {
                    for _ in 0..=num_slice_groups_minus1.0 {
                        pub top_left: UnsignedExpGolombCode;
                        pub bottom_right: UnsignedExpGolombCode;
                    }
                }
                3 | 4 | 5 =>{
                    pub slice_group_change_direction_flag: bool;
                    pub slice_group_change_rate_minus1: UnsignedExpGolombCode;
                }
                6 => {
                    pub pic_size_in_map_units_minus1: UnsignedExpGolombCode;
                    for _ in 0..pic_size_in_map_units_minus1.0 {
                        pub slice_group_id: u8[(num_slice_groups_minus1.0 as f32 + 1f32).log2().ceil() as u8];
                    }
                }
                _ => {}
            }
        }

        pub num_ref_idx_l0_default_active_minus1: UnsignedExpGolombCode;
        pub num_ref_idx_l1_default_active_minus1: UnsignedExpGolombCode;

        pub weighted_pred_flag: bool;
        pub weighted_bipred_idc: u2;
        pub pic_init_qp_minus26: SignedExpGolombCode;
        pub pic_init_qs_minus26: SignedExpGolombCode;
        pub chroma_qp_index_offset: SignedExpGolombCode;

        pub deblocking_filter_control_present_flag: bool;
        pub constrained_intra_pred_flag: bool;
        pub redundant_pic_cnt_present_flag: bool;

        if stream.remaining() != 0 {
            pub transform_8x8_mode_flag: bool;
            pub pic_scaling_matrix_present_flag: bool;

            if pic_scaling_matrix_present_flag {
                for _ in 0..6 {
                    pub scaling_list_4x4: ScalingList[16];
                }

                if transform_8x8_mode_flag {
                    let seq_parameter_set = decoder.find_sequence_parameter_set(seq_parameter_set_id).unwrap();
                    for _ in 0..(if seq_parameter_set.chroma_format_idc != Some(UnsignedExpGolombCode(3)) { 2 } else { 6 }){
                        pub scaling_list_8x8: ScalingList[64];
                    }
                }
            }
        }
    }
}
