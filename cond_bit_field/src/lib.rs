extern crate proc_macro;

use quote::quote;
use syn::parse_macro_input;

mod block;
mod data;
mod expr;
mod r#if;
mod r#match;
mod syn_private;
mod traits;
mod ty;

#[proc_macro]
pub fn cond_bit_field(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let data = parse_macro_input!(input as data::Struct);
  let tokens = quote! {#data};
  tokens.into()
}
