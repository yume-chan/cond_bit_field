#![feature(prelude_import)]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std;
use bit_stream::{cond_bit_field, BitField, BitRead, Result, SizedBitField};

mod exp_golomb {

  // 7.3.2.1.1 Sequence parameter set data syntax

  // TODO

  // 7.3.2.2 Picture parameter set RBSP syntax

  use bit_stream::{BitField, BitRead, Result};
  use num_derive::{FromPrimitive, NumOps, ToPrimitive};
  pub struct UnsignedExpGolombCode(u64);
  #[allow(non_upper_case_globals, unused_qualifications)]
  const _IMPL_NUM_FromPrimitive_FOR_UnsignedExpGolombCode: () = {
    #[allow(clippy::useless_attribute)]
    #[allow(rust_2018_idioms)]
    extern crate num_traits as _num_traits;
    impl _num_traits::FromPrimitive for UnsignedExpGolombCode {
      #[inline]
      fn from_i64(n: i64) -> Option<Self> {
        <u64 as _num_traits::FromPrimitive>::from_i64(n).map(UnsignedExpGolombCode)
      }
      #[inline]
      fn from_u64(n: u64) -> Option<Self> {
        <u64 as _num_traits::FromPrimitive>::from_u64(n).map(UnsignedExpGolombCode)
      }
      #[inline]
      fn from_isize(n: isize) -> Option<Self> {
        <u64 as _num_traits::FromPrimitive>::from_isize(n).map(UnsignedExpGolombCode)
      }
      #[inline]
      fn from_i8(n: i8) -> Option<Self> {
        <u64 as _num_traits::FromPrimitive>::from_i8(n).map(UnsignedExpGolombCode)
      }
      #[inline]
      fn from_i16(n: i16) -> Option<Self> {
        <u64 as _num_traits::FromPrimitive>::from_i16(n).map(UnsignedExpGolombCode)
      }
      #[inline]
      fn from_i32(n: i32) -> Option<Self> {
        <u64 as _num_traits::FromPrimitive>::from_i32(n).map(UnsignedExpGolombCode)
      }
      #[inline]
      fn from_i128(n: i128) -> Option<Self> {
        <u64 as _num_traits::FromPrimitive>::from_i128(n).map(UnsignedExpGolombCode)
      }
      #[inline]
      fn from_usize(n: usize) -> Option<Self> {
        <u64 as _num_traits::FromPrimitive>::from_usize(n).map(UnsignedExpGolombCode)
      }
      #[inline]
      fn from_u8(n: u8) -> Option<Self> {
        <u64 as _num_traits::FromPrimitive>::from_u8(n).map(UnsignedExpGolombCode)
      }
      #[inline]
      fn from_u16(n: u16) -> Option<Self> {
        <u64 as _num_traits::FromPrimitive>::from_u16(n).map(UnsignedExpGolombCode)
      }
      #[inline]
      fn from_u32(n: u32) -> Option<Self> {
        <u64 as _num_traits::FromPrimitive>::from_u32(n).map(UnsignedExpGolombCode)
      }
      #[inline]
      fn from_u128(n: u128) -> Option<Self> {
        <u64 as _num_traits::FromPrimitive>::from_u128(n).map(UnsignedExpGolombCode)
      }
      #[inline]
      fn from_f32(n: f32) -> Option<Self> {
        <u64 as _num_traits::FromPrimitive>::from_f32(n).map(UnsignedExpGolombCode)
      }
      #[inline]
      fn from_f64(n: f64) -> Option<Self> {
        <u64 as _num_traits::FromPrimitive>::from_f64(n).map(UnsignedExpGolombCode)
      }
    }
  };
  impl ::core::ops::Add for UnsignedExpGolombCode {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
      UnsignedExpGolombCode(<u64 as ::core::ops::Add>::add(self.0, other.0))
    }
  }
  impl ::core::ops::Sub for UnsignedExpGolombCode {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
      UnsignedExpGolombCode(<u64 as ::core::ops::Sub>::sub(self.0, other.0))
    }
  }
  impl ::core::ops::Mul for UnsignedExpGolombCode {
    type Output = Self;
    #[inline]
    fn mul(self, other: Self) -> Self {
      UnsignedExpGolombCode(<u64 as ::core::ops::Mul>::mul(self.0, other.0))
    }
  }
  impl ::core::ops::Div for UnsignedExpGolombCode {
    type Output = Self;
    #[inline]
    fn div(self, other: Self) -> Self {
      UnsignedExpGolombCode(<u64 as ::core::ops::Div>::div(self.0, other.0))
    }
  }
  impl ::core::ops::Rem for UnsignedExpGolombCode {
    type Output = Self;
    #[inline]
    fn rem(self, other: Self) -> Self {
      UnsignedExpGolombCode(<u64 as ::core::ops::Rem>::rem(self.0, other.0))
    }
  }
  #[allow(non_upper_case_globals, unused_qualifications)]
  const _IMPL_NUM_ToPrimitive_FOR_UnsignedExpGolombCode: () = {
    #[allow(clippy::useless_attribute)]
    #[allow(rust_2018_idioms)]
    extern crate num_traits as _num_traits;
    impl _num_traits::ToPrimitive for UnsignedExpGolombCode {
      #[inline]
      fn to_i64(&self) -> Option<i64> {
        <u64 as _num_traits::ToPrimitive>::to_i64(&self.0)
      }
      #[inline]
      fn to_u64(&self) -> Option<u64> {
        <u64 as _num_traits::ToPrimitive>::to_u64(&self.0)
      }
      #[inline]
      fn to_isize(&self) -> Option<isize> {
        <u64 as _num_traits::ToPrimitive>::to_isize(&self.0)
      }
      #[inline]
      fn to_i8(&self) -> Option<i8> {
        <u64 as _num_traits::ToPrimitive>::to_i8(&self.0)
      }
      #[inline]
      fn to_i16(&self) -> Option<i16> {
        <u64 as _num_traits::ToPrimitive>::to_i16(&self.0)
      }
      #[inline]
      fn to_i32(&self) -> Option<i32> {
        <u64 as _num_traits::ToPrimitive>::to_i32(&self.0)
      }
      #[inline]
      fn to_i128(&self) -> Option<i128> {
        <u64 as _num_traits::ToPrimitive>::to_i128(&self.0)
      }
      #[inline]
      fn to_usize(&self) -> Option<usize> {
        <u64 as _num_traits::ToPrimitive>::to_usize(&self.0)
      }
      #[inline]
      fn to_u8(&self) -> Option<u8> {
        <u64 as _num_traits::ToPrimitive>::to_u8(&self.0)
      }
      #[inline]
      fn to_u16(&self) -> Option<u16> {
        <u64 as _num_traits::ToPrimitive>::to_u16(&self.0)
      }
      #[inline]
      fn to_u32(&self) -> Option<u32> {
        <u64 as _num_traits::ToPrimitive>::to_u32(&self.0)
      }
      #[inline]
      fn to_u128(&self) -> Option<u128> {
        <u64 as _num_traits::ToPrimitive>::to_u128(&self.0)
      }
      #[inline]
      fn to_f32(&self) -> Option<f32> {
        <u64 as _num_traits::ToPrimitive>::to_f32(&self.0)
      }
      #[inline]
      fn to_f64(&self) -> Option<f64> {
        <u64 as _num_traits::ToPrimitive>::to_f64(&self.0)
      }
    }
  };
  #[automatically_derived]
  #[allow(unused_qualifications)]
  impl ::core::fmt::Debug for UnsignedExpGolombCode {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
      match *self {
        UnsignedExpGolombCode(ref __self_0_0) => {
          let debug_trait_builder =
            &mut ::core::fmt::Formatter::debug_tuple(f, "UnsignedExpGolombCode");
          let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
          ::core::fmt::DebugTuple::finish(debug_trait_builder)
        }
      }
    }
  }
  impl BitField for UnsignedExpGolombCode {
    fn read(reader: &mut impl BitRead) -> Result<Self> {
      let mut length = 0;
      while !reader.read_bit()? {
        length += 1;
      }
      if length == 0 {
        return Ok(UnsignedExpGolombCode(0));
      }
      Ok(UnsignedExpGolombCode(
        1 << length | reader.read_sized::<u64, u8>(length)? - 1,
      ))
    }
  }
  pub struct SignedExpGolombCode(i64);
  #[allow(non_upper_case_globals, unused_qualifications)]
  const _IMPL_NUM_FromPrimitive_FOR_SignedExpGolombCode: () = {
    #[allow(clippy::useless_attribute)]
    #[allow(rust_2018_idioms)]
    extern crate num_traits as _num_traits;
    impl _num_traits::FromPrimitive for SignedExpGolombCode {
      #[inline]
      fn from_i64(n: i64) -> Option<Self> {
        <i64 as _num_traits::FromPrimitive>::from_i64(n).map(SignedExpGolombCode)
      }
      #[inline]
      fn from_u64(n: u64) -> Option<Self> {
        <i64 as _num_traits::FromPrimitive>::from_u64(n).map(SignedExpGolombCode)
      }
      #[inline]
      fn from_isize(n: isize) -> Option<Self> {
        <i64 as _num_traits::FromPrimitive>::from_isize(n).map(SignedExpGolombCode)
      }
      #[inline]
      fn from_i8(n: i8) -> Option<Self> {
        <i64 as _num_traits::FromPrimitive>::from_i8(n).map(SignedExpGolombCode)
      }
      #[inline]
      fn from_i16(n: i16) -> Option<Self> {
        <i64 as _num_traits::FromPrimitive>::from_i16(n).map(SignedExpGolombCode)
      }
      #[inline]
      fn from_i32(n: i32) -> Option<Self> {
        <i64 as _num_traits::FromPrimitive>::from_i32(n).map(SignedExpGolombCode)
      }
      #[inline]
      fn from_i128(n: i128) -> Option<Self> {
        <i64 as _num_traits::FromPrimitive>::from_i128(n).map(SignedExpGolombCode)
      }
      #[inline]
      fn from_usize(n: usize) -> Option<Self> {
        <i64 as _num_traits::FromPrimitive>::from_usize(n).map(SignedExpGolombCode)
      }
      #[inline]
      fn from_u8(n: u8) -> Option<Self> {
        <i64 as _num_traits::FromPrimitive>::from_u8(n).map(SignedExpGolombCode)
      }
      #[inline]
      fn from_u16(n: u16) -> Option<Self> {
        <i64 as _num_traits::FromPrimitive>::from_u16(n).map(SignedExpGolombCode)
      }
      #[inline]
      fn from_u32(n: u32) -> Option<Self> {
        <i64 as _num_traits::FromPrimitive>::from_u32(n).map(SignedExpGolombCode)
      }
      #[inline]
      fn from_u128(n: u128) -> Option<Self> {
        <i64 as _num_traits::FromPrimitive>::from_u128(n).map(SignedExpGolombCode)
      }
      #[inline]
      fn from_f32(n: f32) -> Option<Self> {
        <i64 as _num_traits::FromPrimitive>::from_f32(n).map(SignedExpGolombCode)
      }
      #[inline]
      fn from_f64(n: f64) -> Option<Self> {
        <i64 as _num_traits::FromPrimitive>::from_f64(n).map(SignedExpGolombCode)
      }
    }
  };
  impl ::core::ops::Add for SignedExpGolombCode {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
      SignedExpGolombCode(<i64 as ::core::ops::Add>::add(self.0, other.0))
    }
  }
  impl ::core::ops::Sub for SignedExpGolombCode {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
      SignedExpGolombCode(<i64 as ::core::ops::Sub>::sub(self.0, other.0))
    }
  }
  impl ::core::ops::Mul for SignedExpGolombCode {
    type Output = Self;
    #[inline]
    fn mul(self, other: Self) -> Self {
      SignedExpGolombCode(<i64 as ::core::ops::Mul>::mul(self.0, other.0))
    }
  }
  impl ::core::ops::Div for SignedExpGolombCode {
    type Output = Self;
    #[inline]
    fn div(self, other: Self) -> Self {
      SignedExpGolombCode(<i64 as ::core::ops::Div>::div(self.0, other.0))
    }
  }
  impl ::core::ops::Rem for SignedExpGolombCode {
    type Output = Self;
    #[inline]
    fn rem(self, other: Self) -> Self {
      SignedExpGolombCode(<i64 as ::core::ops::Rem>::rem(self.0, other.0))
    }
  }
  #[allow(non_upper_case_globals, unused_qualifications)]
  const _IMPL_NUM_ToPrimitive_FOR_SignedExpGolombCode: () = {
    #[allow(clippy::useless_attribute)]
    #[allow(rust_2018_idioms)]
    extern crate num_traits as _num_traits;
    impl _num_traits::ToPrimitive for SignedExpGolombCode {
      #[inline]
      fn to_i64(&self) -> Option<i64> {
        <i64 as _num_traits::ToPrimitive>::to_i64(&self.0)
      }
      #[inline]
      fn to_u64(&self) -> Option<u64> {
        <i64 as _num_traits::ToPrimitive>::to_u64(&self.0)
      }
      #[inline]
      fn to_isize(&self) -> Option<isize> {
        <i64 as _num_traits::ToPrimitive>::to_isize(&self.0)
      }
      #[inline]
      fn to_i8(&self) -> Option<i8> {
        <i64 as _num_traits::ToPrimitive>::to_i8(&self.0)
      }
      #[inline]
      fn to_i16(&self) -> Option<i16> {
        <i64 as _num_traits::ToPrimitive>::to_i16(&self.0)
      }
      #[inline]
      fn to_i32(&self) -> Option<i32> {
        <i64 as _num_traits::ToPrimitive>::to_i32(&self.0)
      }
      #[inline]
      fn to_i128(&self) -> Option<i128> {
        <i64 as _num_traits::ToPrimitive>::to_i128(&self.0)
      }
      #[inline]
      fn to_usize(&self) -> Option<usize> {
        <i64 as _num_traits::ToPrimitive>::to_usize(&self.0)
      }
      #[inline]
      fn to_u8(&self) -> Option<u8> {
        <i64 as _num_traits::ToPrimitive>::to_u8(&self.0)
      }
      #[inline]
      fn to_u16(&self) -> Option<u16> {
        <i64 as _num_traits::ToPrimitive>::to_u16(&self.0)
      }
      #[inline]
      fn to_u32(&self) -> Option<u32> {
        <i64 as _num_traits::ToPrimitive>::to_u32(&self.0)
      }
      #[inline]
      fn to_u128(&self) -> Option<u128> {
        <i64 as _num_traits::ToPrimitive>::to_u128(&self.0)
      }
      #[inline]
      fn to_f32(&self) -> Option<f32> {
        <i64 as _num_traits::ToPrimitive>::to_f32(&self.0)
      }
      #[inline]
      fn to_f64(&self) -> Option<f64> {
        <i64 as _num_traits::ToPrimitive>::to_f64(&self.0)
      }
    }
  };
  #[automatically_derived]
  #[allow(unused_qualifications)]
  impl ::core::fmt::Debug for SignedExpGolombCode {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
      match *self {
        SignedExpGolombCode(ref __self_0_0) => {
          let debug_trait_builder =
            &mut ::core::fmt::Formatter::debug_tuple(f, "SignedExpGolombCode");
          let _ = ::core::fmt::DebugTuple::field(debug_trait_builder, &&(*__self_0_0));
          ::core::fmt::DebugTuple::finish(debug_trait_builder)
        }
      }
    }
  }
  impl BitField for SignedExpGolombCode {
    fn read(reader: &mut impl BitRead) -> Result<Self> {
      let ue = reader.read::<UnsignedExpGolombCode>()?.0;
      Ok(Self(if ue % 1 == 1 {
        (ue >> 1 + 1) as i64
      } else {
        ((ue >> 1) as i64) * -1
      }))
    }
  }
}
pub use exp_golomb::*;
pub struct ScalingList {
  pub list: Vec<u8>,
  pub use_default_scaling_matrix_flag: bool,
}
impl SizedBitField for ScalingList {
  fn read_sized(reader: &mut impl BitRead, size: u8) -> Result<Self> {
    let mut list: Vec<u8> = Vec::with_capacity(size as usize);
    let mut last_scale = {
      let delta_scale: SignedExpGolombCode = reader.read()?;
      ((8 + delta_scale.0 + 256) % 256) as u8
    };
    if last_scale == 0 {
      for _ in 0..size {
        list.push(8);
      }
      return Ok(Self {
        list,
        use_default_scaling_matrix_flag: true,
      });
    }
    let mut j = 1u8;
    loop {
      let delta_scale: SignedExpGolombCode = reader.read()?;
      let next_scale: u8 = ((last_scale as i64 + delta_scale.0 + 256) % 256) as u8;
      j += 1;
      if next_scale != 0 {
        last_scale = next_scale;
        list.push(last_scale);
      } else {
        list.push(last_scale);
        break;
      }
      if j == size {
        return Ok(Self {
          list,
          use_default_scaling_matrix_flag: false,
        });
      }
    }
    while j < size {
      list.push(last_scale);
    }
    Ok(Self {
      list,
      use_default_scaling_matrix_flag: false,
    })
  }
}
pub struct SequenceParameterSet {
  pub profile_idc: u8,
  pub constraint_set0_flag: bool,
  pub constraint_set1_flag: bool,
  pub constraint_set2_flag: bool,
  pub constraint_set3_flag: bool,
  pub constraint_set4_flag: bool,
  pub constraint_set5_flag: bool,
  pub level_idc: u8,
  pub seq_parameter_set_id: UnsignedExpGolombCode,
  pub chroma_format_idc: std::option::Option<UnsignedExpGolombCode>,
  pub separate_colour_plane_flag: std::option::Option<bool>,
  pub bit_depth_luma_minus8: std::option::Option<UnsignedExpGolombCode>,
  pub bit_depth_chroma_minus8: std::option::Option<UnsignedExpGolombCode>,
  pub qpprime_y_zero_transform_bypass_flag: std::option::Option<bool>,
  pub seq_scaling_matrix_present_flag: std::option::Option<bool>,
  pub scaling_list_4x4: std::option::Option<std::vec::Vec<ScalingList>>,
  pub scaling_list_8x8: std::option::Option<std::vec::Vec<ScalingList>>,
  pub log2_max_frame_num_minus4: UnsignedExpGolombCode,
  pub pic_order_cnt_type: UnsignedExpGolombCode,
  pub log2_max_pic_order_cnt_lsb_minus4: std::option::Option<UnsignedExpGolombCode>,
  pub delta_pic_order_always_zero_flag: std::option::Option<bool>,
  pub offset_for_non_ref_pic: std::option::Option<SignedExpGolombCode>,
  pub offset_for_top_to_bottom_field: std::option::Option<SignedExpGolombCode>,
  pub num_ref_frames_in_pic_order_cnt_cycle: std::option::Option<UnsignedExpGolombCode>,
  pub offset_for_ref_frame: std::option::Option<std::vec::Vec<SignedExpGolombCode>>,
  pub max_num_ref_frames: UnsignedExpGolombCode,
  pub gaps_in_frame_num_value_allowed_flag: bool,
  pub pic_width_in_mbs_minus1: UnsignedExpGolombCode,
  pub pic_height_in_map_units_minus1: UnsignedExpGolombCode,
  pub frame_mbs_only_flag: bool,
  pub mb_adaptive_frame_field_flag: std::option::Option<bool>,
  pub direct_8x8_inference_flag: bool,
  pub frame_cropping_flag: bool,
  pub frame_crop_left_offset: std::option::Option<UnsignedExpGolombCode>,
  pub frame_crop_right_offset: std::option::Option<UnsignedExpGolombCode>,
  pub frame_crop_top_offset: std::option::Option<UnsignedExpGolombCode>,
  pub frame_crop_bottom_offset: std::option::Option<UnsignedExpGolombCode>,
  pub vui_parameters_present_flag: bool,
}
impl ::bit_stream::BitField for SequenceParameterSet {
  fn read(reader: &mut impl ::bit_stream::BitRead) -> ::bit_stream::Result<Self> {
    let profile_idc: u8 = reader.read_sized(8u8)?;
    let constraint_set0_flag: bool = reader.read_bit()?;
    let constraint_set1_flag: bool = reader.read_bit()?;
    let constraint_set2_flag: bool = reader.read_bit()?;
    let constraint_set3_flag: bool = reader.read_bit()?;
    let constraint_set4_flag: bool = reader.read_bit()?;
    let constraint_set5_flag: bool = reader.read_bit()?;
    reader.skip(2)?;
    let level_idc: u8 = reader.read_sized(8u8)?;
    let seq_parameter_set_id: UnsignedExpGolombCode = reader.read()?;
    #[allow(non_snake_case)]
    let mut qpprime_y_zero_transform_bypass_flag_8N9l: std::option::Option<bool> = None;
    #[allow(non_snake_case)]
    let mut bit_depth_luma_minus8_UPBn: std::option::Option<UnsignedExpGolombCode> = None;
    #[allow(non_snake_case)]
    let mut scaling_list_4x4_JkKV: std::option::Option<std::vec::Vec<ScalingList>> = None;
    #[allow(non_snake_case)]
    let mut seq_scaling_matrix_present_flag_2HKl: std::option::Option<bool> = None;
    #[allow(non_snake_case)]
    let mut separate_colour_plane_flag_6rHL: std::option::Option<bool> = None;
    #[allow(non_snake_case)]
    let mut scaling_list_8x8_oGAa: std::option::Option<std::vec::Vec<ScalingList>> = None;
    #[allow(non_snake_case)]
    let mut chroma_format_idc_GvTB: std::option::Option<UnsignedExpGolombCode> = None;
    #[allow(non_snake_case)]
    let mut bit_depth_chroma_minus8_1p7t: std::option::Option<UnsignedExpGolombCode> = None;
    if profile_idc == 100
      || profile_idc == 110
      || profile_idc == 122
      || profile_idc == 244
      || profile_idc == 44
      || profile_idc == 83
      || profile_idc == 86
      || profile_idc == 118
      || profile_idc == 128
      || profile_idc == 138
      || profile_idc == 139
      || profile_idc == 134
      || profile_idc == 135
    {
      let chroma_format_idc: UnsignedExpGolombCode = reader.read()?;
      #[allow(non_snake_case)]
      let mut separate_colour_plane_flag_nd8R: std::option::Option<bool> = None;
      if chroma_format_idc.0 == 3 {
        let separate_colour_plane_flag: bool = reader.read_bit()?;
        separate_colour_plane_flag_nd8R = Some(separate_colour_plane_flag);
      }
      let separate_colour_plane_flag = separate_colour_plane_flag_nd8R;
      let bit_depth_luma_minus8: UnsignedExpGolombCode = reader.read()?;
      let bit_depth_chroma_minus8: UnsignedExpGolombCode = reader.read()?;
      let qpprime_y_zero_transform_bypass_flag: bool = reader.read_bit()?;
      let seq_scaling_matrix_present_flag: bool = reader.read_bit()?;
      #[allow(non_snake_case)]
      let mut scaling_list_4x4_vqwJ: std::option::Option<std::vec::Vec<ScalingList>> = None;
      #[allow(non_snake_case)]
      let mut scaling_list_8x8_0KCJ: std::option::Option<std::vec::Vec<ScalingList>> = None;
      if seq_scaling_matrix_present_flag {
        #[allow(non_snake_case)]
        let mut scaling_list_4x4_Vc7k: std::vec::Vec<ScalingList> = std::vec::Vec::new();
        for _ in 0..6 {
          let scaling_list_4x4: ScalingList = reader.read_sized(16)?;
          scaling_list_4x4_Vc7k.push(scaling_list_4x4);
        }
        let scaling_list_4x4 = scaling_list_4x4_Vc7k;
        #[allow(non_snake_case)]
        let mut scaling_list_8x8_jnNh: std::vec::Vec<ScalingList> = std::vec::Vec::new();
        for _ in 0..(if chroma_format_idc.0 != 3 { 2 } else { 6 }) {
          let scaling_list_8x8: ScalingList = reader.read_sized(64)?;
          scaling_list_8x8_jnNh.push(scaling_list_8x8);
        }
        let scaling_list_8x8 = scaling_list_8x8_jnNh;
        scaling_list_4x4_vqwJ = Some(scaling_list_4x4);
        scaling_list_8x8_0KCJ = Some(scaling_list_8x8);
      }
      let scaling_list_4x4 = scaling_list_4x4_vqwJ;
      let scaling_list_8x8 = scaling_list_8x8_0KCJ;
      chroma_format_idc_GvTB = Some(chroma_format_idc);
      separate_colour_plane_flag_6rHL = separate_colour_plane_flag;
      bit_depth_luma_minus8_UPBn = Some(bit_depth_luma_minus8);
      bit_depth_chroma_minus8_1p7t = Some(bit_depth_chroma_minus8);
      qpprime_y_zero_transform_bypass_flag_8N9l = Some(qpprime_y_zero_transform_bypass_flag);
      seq_scaling_matrix_present_flag_2HKl = Some(seq_scaling_matrix_present_flag);
      scaling_list_4x4_JkKV = scaling_list_4x4;
      scaling_list_8x8_oGAa = scaling_list_8x8;
    }
    let qpprime_y_zero_transform_bypass_flag = qpprime_y_zero_transform_bypass_flag_8N9l;
    let bit_depth_luma_minus8 = bit_depth_luma_minus8_UPBn;
    let scaling_list_4x4 = scaling_list_4x4_JkKV;
    let seq_scaling_matrix_present_flag = seq_scaling_matrix_present_flag_2HKl;
    let separate_colour_plane_flag = separate_colour_plane_flag_6rHL;
    let scaling_list_8x8 = scaling_list_8x8_oGAa;
    let chroma_format_idc = chroma_format_idc_GvTB;
    let bit_depth_chroma_minus8 = bit_depth_chroma_minus8_1p7t;
    let log2_max_frame_num_minus4: UnsignedExpGolombCode = reader.read()?;
    let pic_order_cnt_type: UnsignedExpGolombCode = reader.read()?;
    #[allow(non_snake_case)]
    let mut delta_pic_order_always_zero_flag_DHIq: std::option::Option<bool> = None;
    #[allow(non_snake_case)]
    let mut num_ref_frames_in_pic_order_cnt_cycle_4EvH: std::option::Option<
      UnsignedExpGolombCode,
    > = None;
    #[allow(non_snake_case)]
    let mut offset_for_non_ref_pic_9ltp: std::option::Option<SignedExpGolombCode> = None;
    #[allow(non_snake_case)]
    let mut log2_max_pic_order_cnt_lsb_minus4_aUuD: std::option::Option<UnsignedExpGolombCode> =
      None;
    #[allow(non_snake_case)]
    let mut offset_for_top_to_bottom_field_hNwq: std::option::Option<SignedExpGolombCode> = None;
    #[allow(non_snake_case)]
    let mut offset_for_ref_frame_U4Np: std::option::Option<std::vec::Vec<SignedExpGolombCode>> =
      None;
    match pic_order_cnt_type.0 {
      0 => {
        let log2_max_pic_order_cnt_lsb_minus4: UnsignedExpGolombCode = reader.read()?;
        log2_max_pic_order_cnt_lsb_minus4_aUuD = Some(log2_max_pic_order_cnt_lsb_minus4);
      }
      1 => {
        let delta_pic_order_always_zero_flag: bool = reader.read_bit()?;
        let offset_for_non_ref_pic: SignedExpGolombCode = reader.read()?;
        let offset_for_top_to_bottom_field: SignedExpGolombCode = reader.read()?;
        let num_ref_frames_in_pic_order_cnt_cycle: UnsignedExpGolombCode = reader.read()?;
        #[allow(non_snake_case)]
        let mut offset_for_ref_frame_FDqQ: std::vec::Vec<SignedExpGolombCode> =
          std::vec::Vec::new();
        for _ in 0..num_ref_frames_in_pic_order_cnt_cycle.0 {
          let offset_for_ref_frame: SignedExpGolombCode = reader.read()?;
          offset_for_ref_frame_FDqQ.push(offset_for_ref_frame);
        }
        let offset_for_ref_frame = offset_for_ref_frame_FDqQ;
        delta_pic_order_always_zero_flag_DHIq = Some(delta_pic_order_always_zero_flag);
        offset_for_non_ref_pic_9ltp = Some(offset_for_non_ref_pic);
        offset_for_top_to_bottom_field_hNwq = Some(offset_for_top_to_bottom_field);
        num_ref_frames_in_pic_order_cnt_cycle_4EvH = Some(num_ref_frames_in_pic_order_cnt_cycle);
        offset_for_ref_frame_U4Np = Some(offset_for_ref_frame);
      }
      _ => {}
    };
    let delta_pic_order_always_zero_flag = delta_pic_order_always_zero_flag_DHIq;
    let num_ref_frames_in_pic_order_cnt_cycle = num_ref_frames_in_pic_order_cnt_cycle_4EvH;
    let offset_for_non_ref_pic = offset_for_non_ref_pic_9ltp;
    let log2_max_pic_order_cnt_lsb_minus4 = log2_max_pic_order_cnt_lsb_minus4_aUuD;
    let offset_for_top_to_bottom_field = offset_for_top_to_bottom_field_hNwq;
    let offset_for_ref_frame = offset_for_ref_frame_U4Np;
    let max_num_ref_frames: UnsignedExpGolombCode = reader.read()?;
    let gaps_in_frame_num_value_allowed_flag: bool = reader.read_bit()?;
    let pic_width_in_mbs_minus1: UnsignedExpGolombCode = reader.read()?;
    let pic_height_in_map_units_minus1: UnsignedExpGolombCode = reader.read()?;
    let frame_mbs_only_flag: bool = reader.read_bit()?;
    #[allow(non_snake_case)]
    let mut mb_adaptive_frame_field_flag_WyGN: std::option::Option<bool> = None;
    if !frame_mbs_only_flag {
      let mb_adaptive_frame_field_flag: bool = reader.read_bit()?;
      mb_adaptive_frame_field_flag_WyGN = Some(mb_adaptive_frame_field_flag);
    }
    let mb_adaptive_frame_field_flag = mb_adaptive_frame_field_flag_WyGN;
    let direct_8x8_inference_flag: bool = reader.read_bit()?;
    let frame_cropping_flag: bool = reader.read_bit()?;
    #[allow(non_snake_case)]
    let mut frame_crop_bottom_offset_B0O7: std::option::Option<UnsignedExpGolombCode> = None;
    #[allow(non_snake_case)]
    let mut frame_crop_left_offset_zfjk: std::option::Option<UnsignedExpGolombCode> = None;
    #[allow(non_snake_case)]
    let mut frame_crop_top_offset_lMhf: std::option::Option<UnsignedExpGolombCode> = None;
    #[allow(non_snake_case)]
    let mut frame_crop_right_offset_dTaW: std::option::Option<UnsignedExpGolombCode> = None;
    if frame_cropping_flag {
      let frame_crop_left_offset: UnsignedExpGolombCode = reader.read()?;
      let frame_crop_right_offset: UnsignedExpGolombCode = reader.read()?;
      let frame_crop_top_offset: UnsignedExpGolombCode = reader.read()?;
      let frame_crop_bottom_offset: UnsignedExpGolombCode = reader.read()?;
      frame_crop_left_offset_zfjk = Some(frame_crop_left_offset);
      frame_crop_right_offset_dTaW = Some(frame_crop_right_offset);
      frame_crop_top_offset_lMhf = Some(frame_crop_top_offset);
      frame_crop_bottom_offset_B0O7 = Some(frame_crop_bottom_offset);
    }
    let frame_crop_bottom_offset = frame_crop_bottom_offset_B0O7;
    let frame_crop_left_offset = frame_crop_left_offset_zfjk;
    let frame_crop_top_offset = frame_crop_top_offset_lMhf;
    let frame_crop_right_offset = frame_crop_right_offset_dTaW;
    let vui_parameters_present_flag: bool = reader.read_bit()?;
    if vui_parameters_present_flag {}
    Ok(SequenceParameterSet {
      profile_idc,
      constraint_set0_flag,
      constraint_set1_flag,
      constraint_set2_flag,
      constraint_set3_flag,
      constraint_set4_flag,
      constraint_set5_flag,
      level_idc,
      seq_parameter_set_id,
      chroma_format_idc,
      separate_colour_plane_flag,
      bit_depth_luma_minus8,
      bit_depth_chroma_minus8,
      qpprime_y_zero_transform_bypass_flag,
      seq_scaling_matrix_present_flag,
      scaling_list_4x4,
      scaling_list_8x8,
      log2_max_frame_num_minus4,
      pic_order_cnt_type,
      log2_max_pic_order_cnt_lsb_minus4,
      delta_pic_order_always_zero_flag,
      offset_for_non_ref_pic,
      offset_for_top_to_bottom_field,
      num_ref_frames_in_pic_order_cnt_cycle,
      offset_for_ref_frame,
      max_num_ref_frames,
      gaps_in_frame_num_value_allowed_flag,
      pic_width_in_mbs_minus1,
      pic_height_in_map_units_minus1,
      frame_mbs_only_flag,
      mb_adaptive_frame_field_flag,
      direct_8x8_inference_flag,
      frame_cropping_flag,
      frame_crop_left_offset,
      frame_crop_right_offset,
      frame_crop_top_offset,
      frame_crop_bottom_offset,
      vui_parameters_present_flag,
    })
  }
}
pub struct PictureParameterSet {
  pub pic_parameter_set_id: UnsignedExpGolombCode,
  pub seq_parameter_set_id: UnsignedExpGolombCode,
  pub entropy_coding_mode_flag: bool,
  pub bottom_field_pic_order_in_frame_present_flag: bool,
  pub num_slice_groups_minus1: UnsignedExpGolombCode,
  pub slice_group_map_type: std::option::Option<UnsignedExpGolombCode>,
  pub run_length_minus1: std::option::Option<std::vec::Vec<UnsignedExpGolombCode>>,
  pub top_left: std::option::Option<std::vec::Vec<UnsignedExpGolombCode>>,
  pub bottom_right: std::option::Option<std::vec::Vec<UnsignedExpGolombCode>>,
  pub slice_group_change_direction_flag: std::option::Option<bool>,
  pub slice_group_change_rate_minus1: std::option::Option<UnsignedExpGolombCode>,
  pub pic_size_in_map_units_minus1: std::option::Option<UnsignedExpGolombCode>,
  pub slice_group_id: std::option::Option<std::vec::Vec<u8>>,
  pub num_ref_idx_l0_default_active_minus1: UnsignedExpGolombCode,
  pub num_ref_idx_l1_default_active_minus1: UnsignedExpGolombCode,
  pub weighted_pred_flag: bool,
  pub weighted_bipred_idc: u8,
  pub pic_init_qp_minus26: UnsignedExpGolombCode,
  pub pic_init_qs_minus26: UnsignedExpGolombCode,
  pub chroma_qp_index_offset: UnsignedExpGolombCode,
  pub deblocking_filter_control_present_flag: bool,
  pub constrained_intra_pred_flag: bool,
  pub redundant_pic_cnt_present_flag: bool,
}
impl ::bit_stream::BitField for PictureParameterSet {
  fn read(reader: &mut impl ::bit_stream::BitRead) -> ::bit_stream::Result<Self> {
    let pic_parameter_set_id: UnsignedExpGolombCode = reader.read()?;
    let seq_parameter_set_id: UnsignedExpGolombCode = reader.read()?;
    let entropy_coding_mode_flag: bool = reader.read_bit()?;
    let bottom_field_pic_order_in_frame_present_flag: bool = reader.read_bit()?;
    let num_slice_groups_minus1: UnsignedExpGolombCode = reader.read()?;
    #[allow(non_snake_case)]
    let mut top_left_Aznt: std::option::Option<std::vec::Vec<UnsignedExpGolombCode>> = None;
    #[allow(non_snake_case)]
    let mut slice_group_id_b3cF: std::option::Option<std::vec::Vec<u8>> = None;
    #[allow(non_snake_case)]
    let mut pic_size_in_map_units_minus1_I3J4: std::option::Option<UnsignedExpGolombCode> = None;
    #[allow(non_snake_case)]
    let mut slice_group_change_rate_minus1_Zz3Q: std::option::Option<UnsignedExpGolombCode> = None;
    #[allow(non_snake_case)]
    let mut run_length_minus1_z1W7: std::option::Option<std::vec::Vec<UnsignedExpGolombCode>> =
      None;
    #[allow(non_snake_case)]
    let mut slice_group_map_type_5V2s: std::option::Option<UnsignedExpGolombCode> = None;
    #[allow(non_snake_case)]
    let mut bottom_right_6T4Y: std::option::Option<std::vec::Vec<UnsignedExpGolombCode>> = None;
    #[allow(non_snake_case)]
    let mut slice_group_change_direction_flag_jM2p: std::option::Option<bool> = None;
    if num_slice_groups_minus1.0 > 0 {
      let slice_group_map_type: UnsignedExpGolombCode = reader.read()?;
      #[allow(non_snake_case)]
      let mut top_left_Ja72: std::option::Option<std::vec::Vec<UnsignedExpGolombCode>> = None;
      #[allow(non_snake_case)]
      let mut slice_group_change_rate_minus1_U14G: std::option::Option<
        UnsignedExpGolombCode,
      > = None;
      #[allow(non_snake_case)]
      let mut slice_group_change_direction_flag_BNm1: std::option::Option<bool> = None;
      #[allow(non_snake_case)]
      let mut run_length_minus1_2bOn: std::option::Option<
        std::vec::Vec<UnsignedExpGolombCode>,
      > = None;
      #[allow(non_snake_case)]
      let mut bottom_right_Trwr: std::option::Option<std::vec::Vec<UnsignedExpGolombCode>> = None;
      #[allow(non_snake_case)]
      let mut slice_group_id_UE9l: std::option::Option<std::vec::Vec<u8>> = None;
      #[allow(non_snake_case)]
      let mut pic_size_in_map_units_minus1_Ha3H: std::option::Option<UnsignedExpGolombCode> = None;
      match slice_group_map_type.0 {
        0 => {
          #[allow(non_snake_case)]
          let mut run_length_minus1_vPSL: std::vec::Vec<UnsignedExpGolombCode> =
            std::vec::Vec::new();
          for _ in 0..=num_slice_groups_minus1.0 {
            let run_length_minus1: UnsignedExpGolombCode = reader.read()?;
            run_length_minus1_vPSL.push(run_length_minus1);
          }
          let run_length_minus1 = run_length_minus1_vPSL;
          run_length_minus1_2bOn = Some(run_length_minus1);
        }
        2 => {
          #[allow(non_snake_case)]
          let mut top_left_6EDj: std::vec::Vec<UnsignedExpGolombCode> = std::vec::Vec::new();
          #[allow(non_snake_case)]
          let mut bottom_right_V27H: std::vec::Vec<UnsignedExpGolombCode> = std::vec::Vec::new();
          for _ in 0..=num_slice_groups_minus1.0 {
            let top_left: UnsignedExpGolombCode = reader.read()?;
            let bottom_right: UnsignedExpGolombCode = reader.read()?;
            top_left_6EDj.push(top_left);
            bottom_right_V27H.push(bottom_right);
          }
          let top_left = top_left_6EDj;
          let bottom_right = bottom_right_V27H;
          top_left_Ja72 = Some(top_left);
          bottom_right_Trwr = Some(bottom_right);
        }
        3 | 4 | 5 => {
          let slice_group_change_direction_flag: bool = reader.read_bit()?;
          let slice_group_change_rate_minus1: UnsignedExpGolombCode = reader.read()?;
          slice_group_change_direction_flag_BNm1 = Some(slice_group_change_direction_flag);
          slice_group_change_rate_minus1_U14G = Some(slice_group_change_rate_minus1);
        }
        6 => {
          let pic_size_in_map_units_minus1: UnsignedExpGolombCode = reader.read()?;
          #[allow(non_snake_case)]
          let mut slice_group_id_2GFn: std::vec::Vec<u8> = std::vec::Vec::new();
          for _ in 0..pic_size_in_map_units_minus1.0 {
            let slice_group_id: u8 =
              reader.read_sized((num_slice_groups_minus1.0 as f32 + 1f32).log2().ceil() as u8)?;
            slice_group_id_2GFn.push(slice_group_id);
          }
          let slice_group_id = slice_group_id_2GFn;
          pic_size_in_map_units_minus1_Ha3H = Some(pic_size_in_map_units_minus1);
          slice_group_id_UE9l = Some(slice_group_id);
        }
        _ => {}
      };
      let top_left = top_left_Ja72;
      let slice_group_change_rate_minus1 = slice_group_change_rate_minus1_U14G;
      let slice_group_change_direction_flag = slice_group_change_direction_flag_BNm1;
      let run_length_minus1 = run_length_minus1_2bOn;
      let bottom_right = bottom_right_Trwr;
      let slice_group_id = slice_group_id_UE9l;
      let pic_size_in_map_units_minus1 = pic_size_in_map_units_minus1_Ha3H;
      slice_group_map_type_5V2s = Some(slice_group_map_type);
      run_length_minus1_z1W7 = run_length_minus1;
      top_left_Aznt = top_left;
      bottom_right_6T4Y = bottom_right;
      slice_group_change_direction_flag_jM2p = slice_group_change_direction_flag;
      slice_group_change_rate_minus1_Zz3Q = slice_group_change_rate_minus1;
      pic_size_in_map_units_minus1_I3J4 = pic_size_in_map_units_minus1;
      slice_group_id_b3cF = slice_group_id;
    }
    let top_left = top_left_Aznt;
    let slice_group_id = slice_group_id_b3cF;
    let pic_size_in_map_units_minus1 = pic_size_in_map_units_minus1_I3J4;
    let slice_group_change_rate_minus1 = slice_group_change_rate_minus1_Zz3Q;
    let run_length_minus1 = run_length_minus1_z1W7;
    let slice_group_map_type = slice_group_map_type_5V2s;
    let bottom_right = bottom_right_6T4Y;
    let slice_group_change_direction_flag = slice_group_change_direction_flag_jM2p;
    let num_ref_idx_l0_default_active_minus1: UnsignedExpGolombCode = reader.read()?;
    let num_ref_idx_l1_default_active_minus1: UnsignedExpGolombCode = reader.read()?;
    let weighted_pred_flag: bool = reader.read_bit()?;
    let weighted_bipred_idc: u8 = reader.read_sized(2u8)?;
    let pic_init_qp_minus26: UnsignedExpGolombCode = reader.read()?;
    let pic_init_qs_minus26: UnsignedExpGolombCode = reader.read()?;
    let chroma_qp_index_offset: UnsignedExpGolombCode = reader.read()?;
    let deblocking_filter_control_present_flag: bool = reader.read_bit()?;
    let constrained_intra_pred_flag: bool = reader.read_bit()?;
    let redundant_pic_cnt_present_flag: bool = reader.read_bit()?;
    Ok(PictureParameterSet {
      pic_parameter_set_id,
      seq_parameter_set_id,
      entropy_coding_mode_flag,
      bottom_field_pic_order_in_frame_present_flag,
      num_slice_groups_minus1,
      slice_group_map_type,
      run_length_minus1,
      top_left,
      bottom_right,
      slice_group_change_direction_flag,
      slice_group_change_rate_minus1,
      pic_size_in_map_units_minus1,
      slice_group_id,
      num_ref_idx_l0_default_active_minus1,
      num_ref_idx_l1_default_active_minus1,
      weighted_pred_flag,
      weighted_bipred_idc,
      pic_init_qp_minus26,
      pic_init_qs_minus26,
      chroma_qp_index_offset,
      deblocking_filter_control_present_flag,
      constrained_intra_pred_flag,
      redundant_pic_cnt_present_flag,
    })
  }
}
#[cfg(test)]
mod test {
  extern crate test;
  #[cfg(test)]
  #[rustc_test_marker]
  pub const parse_pic_param_set: test::TestDescAndFn = test::TestDescAndFn {
    desc: test::TestDesc {
      name: test::StaticTestName("test::parse_pic_param_set"),
      ignore: false,
      allow_fail: false,
      should_panic: test::ShouldPanic::No,
      test_type: test::TestType::UnitTest,
    },
    testfn: test::StaticTestFn(|| test::assert_test_result(parse_pic_param_set())),
  };
  fn parse_pic_param_set() {}
}
#[main]
pub fn main() -> () {
  extern crate test;
  test::test_main_static(&[&parse_pic_param_set])
}
