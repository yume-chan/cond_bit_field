# H.264 NAL Unit Parser in Rust

## Packages

* array_fill: A proc macro to initialize large array.
* bit_stream: A simple bit stream suitable for H.264 NAL Unit parsing
* cond_bit_field: A proc macro to generate complex bit field (with conditions, loops, etc.) from extended struct syntax.
* derive_new_number: A proc macro to impl some basic traits for number new types.
* h264_nalu: H.264 NAL Unit parser utilizing cond_bit_field

## Status

My original plan was adding extra variants into `syn::Stmt` enum to support new syntax.

The problem is, by making my own `cond_bit_field::Stmt`, I need to add many (but not all) types to use my `Stmt` instead.

Concisely, I want my `Stmt` is used only to parse the "struct function", which means syntaxes like `ExprIf`, `ExprLoop` etc should use my `Stmt`, while `ItemFn`, `ItemStruct` should not.

Obviously it's not a common use case of `syn`, and because of many functions in `syn` are not exported, a huge amount of code must be duplicated.

A possible solution is folking `syn` and make changes, while also depending on original `syn` for parsing routines that should not use my types.

Anyway, currently the development is halted. The idea sounds simple but it's really difficult to implement and maintain.

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
