extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse::{Parse, ParseStream},
          parse_macro_input, token, Error, Expr, ExprLit, Lit};

struct ExprRepeat {
    pub expr: Expr,
    pub len: usize,
}

impl Parse for ExprRepeat {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr = input.parse()?;
        input.parse::<token::Semi>()?;
        let len_lit: ExprLit = input.parse()?;

        let len = match &len_lit.lit {
            Lit::Int(int) => int.base10_parse::<usize>(),
            _ => Err(Error::new_spanned(
                len_lit,
                "length of repeat expressions must be a number",
            )),
        }?;

        Ok(Self { expr, len })
    }
}

impl ToTokens for ExprRepeat {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        token::Bracket::default().surround(tokens, |tokens| {
            for _ in 0..self.len {
                self.expr.to_tokens(tokens);
                token::Comma::default().to_tokens(tokens);
            }
        });
    }
}

/// Initialize an array by filling it with specified value.
///
/// Normally there are two ways to initialize an array:
///
/// ```
/// let arr = [0; 32];
/// ```
///
/// and
///
/// ```
/// let arr: [u8; 64] = std::default::Default();
/// ```
///
/// However, method 1 only support array up to 32 elements, and
/// method 2 requires the item type to be `Copy`
///
/// This macro, use like
///
/// ```
/// let arr: [Option<Vec<u8>>; 128] = array_fill![None; 128];
/// ```
///
/// will expand to
///
/// ```
/// let arr: [Option<Vec<u8>>; 128] = [None, None, /* ... */, None, None];
/// ```
///
/// So it supports any size (as long as it fits in the stack) and any value.
#[proc_macro]
pub fn array_fill(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let repeat = parse_macro_input! {input as ExprRepeat};
    let mut tokens = TokenStream::new();
    repeat.to_tokens(&mut tokens);
    tokens.into()
}
