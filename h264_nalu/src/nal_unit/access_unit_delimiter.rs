use bit_stream::cond_bit_field;
use serde::Serialize;

// 7.3.2.4 Access unit delimiter RBSP syntax
cond_bit_field! {
    #[derive(Clone, Debug, Serialize)]
    pub struct AccessUnitDelimiter {
        pub primary_pic_type: u3;
    }
}
