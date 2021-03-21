use std::convert::{TryFrom, TryInto};

use bit_stream::{cond_bit_field, BitStreamError};
use serde::Serialize;

use crate::{nal_unit::{NalUnitHeader, SignedExpGolombCode, UnsignedExpGolombCode},
            Decoder};

cond_bit_field! {
    /// slice_header
    ///
    /// § 7.3.3 Slice header syntax
    #[derive(Clone, Debug, Serialize)]
    #[extra_args(decoder: &Decoder, header: &NalUnitHeader)]
    pub struct SliceHeader {
        pub first_mb_in_slice: UnsignedExpGolombCode;
        /// specifies the coding type of the slice according to Table 7-6.
        ///
        /// § 7.4.3 Slice header semantics
        ///
        /// | slice_type | Name of slice_type |
        /// |------------|--------------------|
        /// | 0          | P (P slice)        |
        /// | 1          | B (B slice)        |
        /// | 2          | I (I slice)        |
        /// | 3          | SP (SP slice)      |
        /// | 4          | SI (SI slice)      |
        /// | 5          | P (P slice)        |
        /// | 6          | B (B slice)        |
        /// | 7          | I (I slice)        |
        /// | 8          | SP (SP slice)      |
        /// | 9          | SI (SI slice)      |
        ///
        /// Table 7-6 – Name association to slice_type
        pub slice_type: UnsignedExpGolombCode;
        let slice_type_name: SliceTypeName = slice_type.0.try_into().or(Err(BitStreamError::TooLarge))?;

        pub pic_parameter_set_id: UnsignedExpGolombCode;

        // § 3.148 sequence parameter set
        let pic_parameter_set = decoder.find_picture_parameter_set(pic_parameter_set_id).unwrap();
        let seq_parameter_set = decoder.find_sequence_parameter_set(pic_parameter_set.seq_parameter_set_id).unwrap();
        if seq_parameter_set.separate_colour_plane_flag {
            pub colour_plane_id: u2;
        }

        /// is used as an identifier for pictures and shall be represented by
        /// log2_max_frame_num_minus4 + 4 bits in the bitstream.
        ///
        /// § 7.4.3 Slice header semantics
        #[size(seq_parameter_set.log2_max_frame_num_minus4.0 + 4)]
        pub frame_num: u8;

        if !seq_parameter_set.frame_mbs_only_flag {
            /// equal to 1 specifies that the slice is a slice of a coded field.
            /// field_pic_flag equal to 0 specifies that the slice is a slice of a coded frame.
            /// When field_pic_flag is not present it shall be inferred to be equal to 0.
            ///
            /// § 7.4.3 Slice header semantics
            #[default(false)]
            pub field_pic_flag: bool;
            if field_pic_flag {
                pub bottom_field_flag: bool;
            }
        }

        // In the text, coded slice NAL unit collectively refers to a coded slice of a non-IDR
        // picture NAL unit or to a coded slice of an IDR picture NAL unit. The variable
        // `IdrPicFlag` is specified as
        //
        // ```c
        // IdrPicFlag = ( ( nal_unit_type = = 5 ) ? 1 : 0 )
        // ```
        //
        // § 7.4.1 NAL unit semantics
        #[allow(non_snake_case)]
        let IdrPicFlag = header.ty == 5;

        if IdrPicFlag {
            pub idr_pic_id: UnsignedExpGolombCode;
        }

        if seq_parameter_set.pic_order_cnt_type == 0 {
            /// specifies the picture order count modulo MaxPicOrderCntLsb for
            /// the top field of a coded frame or for a coded field. The length of the
            /// pic_order_cnt_lsb syntax element is log2_max_pic_order_cnt_lsb_minus4 + 4 bits.
            /// The value of the pic_order_cnt_lsb shall be in the range of 0 to
            /// MaxPicOrderCntLsb − 1, inclusive.
            ///
            /// § 7.4.3 Slice header semantics
            #[size(seq_parameter_set.log2_max_pic_order_cnt_lsb_minus4.unwrap().0 + 4)]
            pub pic_order_cnt_lsb: u8;
        }

        if pic_parameter_set.bottom_field_pic_order_in_frame_present_flag && !field_pic_flag {
            pub delta_pic_order_cnt_bottom: SignedExpGolombCode;
        }

        if seq_parameter_set.pic_order_cnt_type == 1 &&
            !seq_parameter_set.delta_pic_order_always_zero_flag.unwrap() {
            pub delta_pic_order_cnt_0: SignedExpGolombCode;

            if pic_parameter_set.bottom_field_pic_order_in_frame_present_flag && !field_pic_flag {
                pub delta_pic_order_cnt_1: SignedExpGolombCode;
            }
        }

        if pic_parameter_set.redundant_pic_cnt_present_flag {
            pub redundant_pic_cnt: UnsignedExpGolombCode;
        }

        if slice_type_name == SliceTypeName::B {
            pub direct_spatial_mv_pred_flag: bool;
        }

        if slice_type_name == SliceTypeName::P ||
            slice_type_name == SliceTypeName::SP ||
            slice_type_name == SliceTypeName::B {
            pub num_ref_idx_active_override_flag: bool;

            if num_ref_idx_active_override_flag {
                pub num_ref_idx_l0_active_minus1: UnsignedExpGolombCode;

                if slice_type_name == SliceTypeName::B {
                    pub num_ref_idx_l1_active_minus1: UnsignedExpGolombCode;
                }
            }
        }

        if header.ty == 20 || header.ty == 21 {
            // TODO ref_pic_list_mvc_modification()
        } else {
            #[extra_args(&slice_type_name)]
            pub ref_pic_list_modification: RefPicListModification;
        }

        if (
            pic_parameter_set.weighted_pred_flag &&
            (slice_type_name == SliceTypeName::P || slice_type_name == SliceTypeName::SP)
        ) || (
            pic_parameter_set.weighted_bipred_idc == 1 &&
            (slice_type_name == SliceTypeName::P)
        ) {
            // TODO pred_weight_table
        }

        if header.ref_idc != 0 {
            // TODO dec_ref_pic_marking
        }

        if pic_parameter_set.entropy_coding_mode_flag &&
            slice_type_name != SliceTypeName::I &&
            slice_type_name != SliceTypeName::SI {
            pub cabac_init_idc: UnsignedExpGolombCode;
        }

        pub slice_qp_delta: SignedExpGolombCode;

        if slice_type_name == SliceTypeName::SP ||
            slice_type_name == SliceTypeName::SI {
            if slice_type_name == SliceTypeName::SP {
                pub sp_for_switch_flag: bool;
            }

            pub slice_qs_delta: SignedExpGolombCode;
        }

        if pic_parameter_set.deblocking_filter_control_present_flag {
            pub disable_deblocking_filter_idc: UnsignedExpGolombCode;

            if disable_deblocking_filter_idc != 1 {
                pub slice_alpha_c0_offset_div2: SignedExpGolombCode;
                pub slice_beta_offset_div2: SignedExpGolombCode;
            }
        }

        if pic_parameter_set.num_slice_groups_minus1 > 0 &&
            pic_parameter_set.slice_group_map_type.unwrap() >= 3 &&
            pic_parameter_set.slice_group_map_type.unwrap() <= 5 {
            #[allow(non_snake_case)]
            let PicWidthInMbs = seq_parameter_set.pic_width_in_mbs_minus1.0 + 1;
            #[allow(non_snake_case)]
            let PicHeightInMapUnits = seq_parameter_set.pic_height_in_map_units_minus1.0 + 1;
            #[allow(non_snake_case)]
            let PicSizeInMapUnits = PicWidthInMbs * PicHeightInMapUnits;
            #[allow(non_snake_case)]
            let SliceGroupChangeRate = pic_parameter_set.slice_group_change_rate_minus1.unwrap().0 + 1;

            /// is used to derive the number of slice group map units in
            /// slice group 0 when slice_group_map_type is equal to 3, 4, or 5, as specified by
            ///
            /// ```c
            /// MapUnitsInSliceGroup0 = Min( slice_group_change_cycle * SliceGroupChangeRate, PicSizeInMapUnits )
            /// ```
            ///
            /// The value of slice_group_change_cycle is represented in the bitstream
            /// by the following number of bits
            ///
            /// ```c
            /// Ceil( Log2( PicSizeInMapUnits ÷ SliceGroupChangeRate + 1 ) )
            /// ```
            ///
            /// The value of slice_group_change_cycle shall be in the range of
            /// 0 to Ceil( PicSizeInMapUnits ÷ SliceGroupChangeRate ), inclusive.
            ///
            /// § 7.4.3 Slice header semantics
            #[size(((PicSizeInMapUnits / SliceGroupChangeRate + 1) as f32).log2().ceil() as u8)]
            pub slice_group_change_cycle: u8;
        }
    }
}

cond_bit_field! {
    /// ref_pic_list_modification
    ///
    /// § 7.3.3.1 Reference picture list modification syntax
    #[derive(Clone, Debug, Serialize)]
    #[extra_args(slice_type_name: &SliceTypeName)]
    pub struct RefPicListModification {
        if slice_type_name != &SliceTypeName::I &&
            slice_type_name != &SliceTypeName::SI {
            pub ref_pic_list_modification_flag_l0: bool;
            if ref_pic_list_modification_flag_l0 {
                // TODO
            }
        }

        if slice_type_name == &SliceTypeName::B {
            pub ref_pic_list_modification_flag_l1: bool;
            if ref_pic_list_modification_flag_l1 {
                // TODO
            }
        }
    }
}

/// | slice_type | Name of slice_type |
/// |------------|--------------------|
/// | 0          | P (P slice)        |
/// | 1          | B (B slice)        |
/// | 2          | I (I slice)        |
/// | 3          | SP (SP slice)      |
/// | 4          | SI (SI slice)      |
/// | 5          | P (P slice)        |
/// | 6          | B (B slice)        |
/// | 7          | I (I slice)        |
/// | 8          | SP (SP slice)      |
/// | 9          | SI (SI slice)      |
///
/// Table 7-6 – Name association to slice_type
#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum SliceTypeName {
    P,
    B,
    I,
    SP,
    SI,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct SliceTypeNameParseError(());

impl TryFrom<u64> for SliceTypeName {
    /// The type returned in the event of a conversion error.
    type Error = SliceTypeNameParseError;

    /// Performs the conversion.
    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 | 5 => Ok(SliceTypeName::P),
            1 | 6 => Ok(SliceTypeName::B),
            2 | 7 => Ok(SliceTypeName::I),
            3 | 8 => Ok(SliceTypeName::SP),
            4 | 9 => Ok(SliceTypeName::SI),
            _ => Err(SliceTypeNameParseError(())),
        }
    }
}

impl SliceHeader {
    pub fn slice_type_name(&self) -> SliceTypeName {
        self.slice_type.0.try_into().unwrap()
    }
}
