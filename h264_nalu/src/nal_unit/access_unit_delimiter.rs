use bit_stream::cond_bit_field;
use serde::Serialize;

cond_bit_field! {
    /// access_unit_delimiter_rbsp
    ///
    /// § 7.3.2.4 Access unit delimiter RBSP syntax
    #[derive(Clone, Debug, Serialize)]
    pub struct AccessUnitDelimiter {
        pub primary_pic_type: u3;
    }
}
