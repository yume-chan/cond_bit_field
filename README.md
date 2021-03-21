# H.264 NAL Unit Parser in Rust

## Packages

* array_fill: A proc macro to initialize large array.
* bit_stream: A simple bit stream suitable for H.264 NAL Unit parsing
* cond_bit_field: A proc macro to generate complex bit field (with conditions, loops, etc.) with "normal" struct syntax.
* derive_new_number: A proc macro to impl some basic traits for number new types.
* h264_nalu: H.264 NAL Unit parser utilizing cond_bit_field

## Progress

- [x] Basic parser structure
- [x] NAL Unit Header
  - [ ] extensions
- [ ] (type 1-5) Coded slice
  - [x] Header
    - [ ] ref_pic_list_modification (requires while loop)
    - [ ] pred_weight_table
    - [ ] dec_ref_pic_marking
  - [ ] Data (requires mutable state)
  - [ ] Others
- [x] (type 7) Sequence parameter set
- [x] (type 8) Picture parameter set
- [x] (type 9) Access unit delimiter
- [ ] (type 10-16)
- [ ] (type 19-21)
