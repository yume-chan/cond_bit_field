use bit_stream::cond_bit_field;
use serde::Serialize;

use crate::{nal_unit::{NalUnitHeader, SliceHeader},
            Decoder};

cond_bit_field! {
    /// § 7.3.4 Slice data syntax
    #[derive(Clone, Debug, Serialize)]
    #[extra_args(decoder: &Decoder, header: &SliceHeader)]
    pub struct SliceData {
        // § 3.148 sequence parameter set
        let pic_parameter_set = decoder.find_picture_parameter_set(header.pic_parameter_set_id).unwrap();
        let seq_parameter_set = decoder.find_sequence_parameter_set(pic_parameter_set.seq_parameter_set_id).unwrap();

        if pic_parameter_set.entropy_coding_mode_flag {
            // TODO
        }

        // § 7.4.3 Slice header semantics
        #[allow(non_snake_case)]
        let MbaffFrameFlag = seq_parameter_set.mb_adaptive_frame_field_flag && !header.field_pic_flag;
        // #[allow(non_snake_case)]
        // let mut CurrMbAddr = header.first_mb_in_slice;
        if MbaffFrameFlag {
            // CurrMbAddr *= 2;
        }

        // TODO
    }
}

cond_bit_field! {
    /// § 7.3.2.8 Slice layer without partitioning RBSP syntax
    #[derive(Clone, Debug, Serialize)]
    #[extra_args(decoder: &Decoder, header: &NalUnitHeader)]
    pub struct SliceLayerWithoutPartitioning {
        pub slice_header: SliceHeader[decoder, header];
        pub slice_data: SliceData[decoder, &slice_header];
    }
}
