use crate::nal_unit::{ScalingList, SignedExpGolombCode, UnsignedExpGolombCode};
use bit_stream::cond_bit_field;
use serde::Serialize;

cond_bit_field! {
    /// A _syntax structure containing syntax elements_ that apply to zero or more
    /// _layer representations_ with the dependency_id _syntax element_ equal to 0
    /// and the quality_id _syntax element_ equal to 0 as determined by the content of a
    /// seq_parameter_set_id _syntax element_ found in the _picture parameter set_ referred to by
    /// the pic_parameter_set_id _syntax element_ found in each _slice header_ of _I_, _P_, and
    /// _B slices_.
    ///
    /// § 3.115 picture parameter set
    ///
    /// seq_parameter_set_rbsp
    ///
    /// § 7.3.2.1.1 Sequence parameter set data syntax
    #[derive(Clone, Debug, Serialize)]
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

            if chroma_format_idc == 3 {
                /// equal to 1 specifies that the three colour components
                /// of the 4:4:4 chroma format are coded separately.
                /// separate_colour_plane_flag equal to 0 specifies that the colour components
                /// are not coded separately.
                /// When separate_colour_plane_flag is not present,
                /// it shall be inferred to be equal to 0.
                /// When separate_colour_plane_flag is equal to 1, the primary coded picture
                /// consists of three separate components, each of which consists of coded
                /// samples of one colour plane (Y, Cb or Cr) that each use the monochrome coding
                /// syntax.
                /// In this case, each colour plane is associated with a specific
                /// colour_plane_id value.
                ///
                /// § 7.4.2.1.1 Sequence parameter set data semantics
                #[default(false)]
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

                for _ in 0..(if chroma_format_idc != 3 { 2 } else { 6 }){
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
            /// equal to 0 specifies no switching between frame and field macroblocks
            /// within a picture. mb_adaptive_frame_field_flag equal to 1 specifies the possible
            /// use of switching between frame and field macroblocks within frames.
            /// When mb_adaptive_frame_field_flag is not present,
            /// it shall be inferred to be equal to 0.
            ///
            /// § 7.4.2.1.1 Sequence parameter set data semantics
            #[default(false)]
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
            pub yuv_parameters: YuvParameters;
        }
    }
}

#[allow(non_upper_case_globals)]
const Extended_SAR: u8 = 255;

// E.1.1 VUI parameters syntax
cond_bit_field! {
    #[derive(Clone, Debug, Serialize)]
    pub struct YuvParameters {
        pub aspect_ratio_info_present_flag: bool;

        if aspect_ratio_info_present_flag {
            pub aspect_ratio_idc: u8;

            if aspect_ratio_idc == Extended_SAR {
                pub sar_width: u16;
                pub sar_height: u16;
            }
        }

        pub overscan_info_present_flag: bool;
        if overscan_info_present_flag {
            pub overscan_appropriate_flag: bool;
        }

        pub video_signal_type_present_flag: bool;
        if video_signal_type_present_flag {
            pub video_format: u3;
            pub video_full_range_flag: bool;
            pub colour_description_present_flag: bool;

            if colour_description_present_flag {
                pub colour_primaries: u8;
                pub transfer_characteristics: u8;
                pub matrix_coefficients: u8;
            }
        }

        pub chroma_loc_info_present_flag: bool;
        if chroma_loc_info_present_flag {
            pub chroma_sample_loc_type_top_field: UnsignedExpGolombCode;
            pub chroma_sample_loc_type_bottom_field: UnsignedExpGolombCode;
        }

        pub timing_info_present_flag: bool;
        if timing_info_present_flag {
            pub num_units_in_tick: u32;
            pub time_scale: u32;
            pub fixed_frame_rate_flag: bool;
        }

        pub nal_hrd_parameters_present_flag: bool;
        if nal_hrd_parameters_present_flag {
            pub nal_hrd_parameters: HrdParameters;
        }

        pub vcl_hrd_parameters_present_flag: bool;
        if vcl_hrd_parameters_present_flag {
            pub vcl_hrd_parameters: HrdParameters;
        }

        if nal_hrd_parameters_present_flag || vcl_hrd_parameters_present_flag {
            pub low_delay_hrd_flag: bool;
        }

        pub pic_struct_present_flag: bool;

        pub bitstream_restriction_flag: bool;
        if bitstream_restriction_flag {
            pub motion_vectors_over_pic_boundaries_flag: bool;
            pub max_bytes_per_pic_denom: UnsignedExpGolombCode;
            pub max_bits_per_mb_denom: UnsignedExpGolombCode;
            pub log2_max_mv_length_horizontal: UnsignedExpGolombCode;
            pub log2_max_mv_length_vertical: UnsignedExpGolombCode;
            pub max_num_reorder_frames: UnsignedExpGolombCode;
            pub max_dec_frame_buffering: UnsignedExpGolombCode;
        }
    }
}

cond_bit_field! {
    /// § E.1.2 HRD parameters syntax
    #[derive(Clone, Debug, Serialize)]
    pub struct HrdParameters {
        pub cpb_cnt_minus1: UnsignedExpGolombCode;
        pub bit_rate_scale: u4;
        pub cpb_size_scale: u4;

        for _ in 0..cpb_cnt_minus1.0 {
            pub bit_rate_value_minus1: UnsignedExpGolombCode;
            pub cpb_size_value_minus1: UnsignedExpGolombCode;
            pub cbr_flag: bool;
        }

        pub initial_cpb_removal_delay_length_minus1: u5;
        pub cpb_removal_delay_length_minus1: u5;
        pub dpb_output_delay_length_minus1: u5;
        pub time_offset_length: u5;
    }
}
