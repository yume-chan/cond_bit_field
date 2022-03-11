# H.264 NAL Unit Parser in Rust

## Packages

* array_fill: A proc macro to initialize large array.
* bit_stream: A simple bit stream suitable for H.264 NAL Unit parsing
* cond_bit_field: A proc macro to generate complex bit field (with conditions, loops, etc.) from extended struct syntax.
* derive_new_number: A proc macro to impl some basic traits for number new types.
* h264_nalu: H.264 NAL Unit parser utilizing cond_bit_field

## Status

My original plan was adding extra variants into `syn::Stmt` enum to support new syntax.

The problem is, by making my own `cond_bit_field::Stmt`, I also need to alter many (if not all) AST types to use my `Stmt` instead.

Concisely, I want my `Stmt` is only used to parse the "struct function", which means syntaxes like `ExprIf`, `ExprLoop` etc should use my `Stmt`, while `ItemFn`, `ItemStruct` should not.

Obviously it's not a common use case of `syn`, and because of many functions in `syn` are not exported, a huge amount of code must be copy-pasted.

A possible solution is folking `syn` and make changes, while also depending on original `syn` for parsing routines that should not use my types.

Anyway, currently the development is halted. The idea sounds simple but it's really difficult to implement and maintain.

## Status V2

The new idea is:

1. The syntax must be valid Rust code
2. Use some special syntax to identify the "struct function"

So, as it's a function, it maybe look like:

```rust
#[struct]
fn StructName(arg1: Type1, arg2: Type2) { // no return type
    // `3` is not a valid type,
    // so use `u3` instead.
    // can only be unsigned, but allow any number of bits
    // still round up to nearest power of 2 as real type
    #[field]
    let field_a: u3;

    // normal varaible are still possible
    let temp = arg1;

    if arg2 {
        // fields can be nested
        // will transform to `Option<u16>` in final struct.
        #[field]
        let field_b: u15;
    }

    for i in 0..16 {
        // fields in iteration
        // will transform to `Vec<u8>`
        #[field]
        let field_e: u5;
    }

    // `bool` is 1 bit
    // still `bool` in final struct
    #[field]
    let field_c: bool;

    // field values can also be used as normal variables
    if field_c {
        #[field]
        let field_d: bool;
    }

    // fields in `if` can be used outside
    // but the type is `Option<T>` now
    if let Some(_) = field_d {
        for i in 0..16 {
            // `if` and `for` can be nested freely
            // each time it will be nested in `Option<T>` or `Vec<T>` more deeply
            #[field]
            let field_e: u5;
        }
    }

    if arg1 {
        if arg2 {
            // won't be `Option<Option<T>>`, collapses to one level of `Option<T>`
            #[field]
            let field_f: u5;
        }
    }
}
```

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
