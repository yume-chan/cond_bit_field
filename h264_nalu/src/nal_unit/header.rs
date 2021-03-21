use bit_stream::cond_bit_field;
use serde::Serialize;

cond_bit_field! {
    // § 7.3.1 NAL unit syntax
    #[derive(Clone, Copy, Debug, Serialize)]
    pub struct NalUnitHeader {
        _: 1;

        pub ref_idc: u2;
        /// specifies the type of RBSP data structure contained in the NAL unit as specified in Table 7-1
        ///
        /// § 7.4.1 NAL unit semantics
        ///
        /// | nal_unit_type | Content of NAL                                                                      | RBSP syntax structure                   |
        /// |---------------|-------------------------------------------------------------------------------------|-----------------------------------------|
        /// | 0             | Unspecified                                                                         |                                         |
        /// | 1             | Coded slice of a non-IDR picture                                                    | slice_layer_without_partitioning_rbsp() |
        /// | 2             | Coded slice data partition A                                                        | slice_data_partition_a_layer_rbsp()     |
        /// | 3             | Coded slice data partition B                                                        | slice_data_partition_b_layer_rbsp()     |
        /// | 4             | Coded slice data partition C                                                        | slice_data_partition_c_layer_rbsp()     |
        /// | 5             | Coded slice of an IDR picture                                                       | slice_layer_without_partitioning_rbsp() |
        /// | 6             | Supplemental enhancement information (SEI)                                          | sei_rbsp()                              |
        /// | 7             | Sequence parameter set                                                              | seq_parameter_set_rbsp()                |
        /// | 8             | Picture parameter set                                                               | pic_parameter_set_rbsp()                |
        /// | 9             | Access unit delimiter                                                               | access_unit_delimiter_rbsp()            |
        /// | 10            | End of sequence                                                                     | end_of_seq_rbsp()                       |
        /// | 11            | End of stream                                                                       | end_of_stream_rbsp()                    |
        /// | 12            | Filler data                                                                         | filler_data_rbsp()                      |
        /// | 13            | Sequence parameter set extension                                                    | seq_parameter_set_extension_rbsp()      |
        /// | 14            | Prefix NAL unit                                                                     | prefix_nal_unit_rbsp()                  |
        /// | 15            | Subset sequence parameter set                                                       | subset_seq_parameter_set_rbsp()         |
        /// | 16            | Depth parameter set                                                                 | depth_parameter_set_rbsp()              |
        /// | 17..18        | Reserved                                                                            |                                         |
        /// | 19            | Coded slice of an auxiliary coded picture without partitioning                      | slice_layer_without_partitioning_rbsp() |
        /// | 20            | Coded slice extension                                                               | slice_layer_extension_rbsp()            |
        /// | 21            | Coded slice extension for a depth view component or a 3D-AVC texture view component | slice_layer_extension_rbsp()            |
        /// | 22..23        | Reserved                                                                            |                                         |
        /// | 24..31        | Unspecified                                                                         |                                         |
        ///
        /// Table 7-1 – NAL unit type codes, syntax element categories, and NAL unit type classes
        pub ty: u5;

        match ty {
            14 | 20 | 21 => {
                if ty != 21 {
                    /// indicates whether a nal_unit_header_svc_extension( )
                    /// or nal_unit_header_mvc_extension( ) will follow next in the syntax structure.
                    ///
                    /// When svc_extension_flag is not present, the value of svc_extension_flag is
                    /// inferred to be equal to 0.
                    ///
                    /// § 7.4.1 NAL unit semantics
                    #[default(false)]
                    pub svc_extension_flag: bool;
                } else {
                    /// indicates for NAL units having nal_unit_type equal
                    /// to 21 whether a nal_unit_header_mvc_extension( ) or
                    /// nal_unit_header_3davc_extension( ) will follow next in the syntax structure.
                    ///
                    /// When avc_3d_extension_flag is not present, the value of
                    /// avc_3d_extension_flag is inferred to be equal to 0.
                    ///
                    /// § 7.4.1 NAL unit semantics
                    #[default(false)]
                    pub avc_3d_extension_flag: bool;
                }

                if svc_extension_flag {
                    // TODO nal_unit_header_svc_extension
                } else if avc_3d_extension_flag {
                    // TODO nal_unit_header_3davc_extension
                } else {
                    // TODO nal_unit_header_mvc_extension
                }
            },
            _ => {}
        }
    }
}
