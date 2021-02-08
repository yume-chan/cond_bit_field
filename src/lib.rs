extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{braced, token, Attribute, Expr, ExprIf, Ident, Result, Token, Visibility};

struct NalUnitStruct {
  attrs: Vec<Attribute>,
  visibility: Visibility,
  struct_token: Token![struct],
  name: Ident,
  brace_token: token::Brace,
  item: Vec<NalUnitItem>,
}

struct NalUnitField {
  let_token: token::Let,
  name: Ident,
  eq_token: token::Eq,
  length: Box<Expr>,
}

enum NalUnitItem {
  Field(NalUnitField),
  If(ExprIf),
}

impl Parse for NalUnitStruct {
  fn parse(input: ParseStream) -> Result<Self> {
    let content;
    Ok(NalUnitStruct {
      attrs: input.call(Attribute::parse_outer)?,
      visibility: input.parse()?,
      struct_token: input.parse()?,
      name: input.parse()?,
      brace_token: braced!(content in input),
      item: vec![],
    })
  }
}

impl Parse for NalUnitItem {
  fn parse(input: ParseStream) -> Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(Token![if]) {
      let if_expr: ExprIf = input.parse()?;
      Ok(NalUnitItem::If(input.parse()?))
    } else if lookahead.peek(Token![let]) {
      input.parse().map(NalUnitItem::Field)
    } else {
      Err(lookahead.error())
    }
  }
}

impl Parse for NalUnitField {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(NalUnitField {
      let_token: input.parse()?,
      name: input.parse()?,
      eq_token: input.parse()?,
      length: input.parse()?,
    })
  }
}

#[proc_macro]
pub fn nal_unit_def(input: TokenStream) -> TokenStream {
  let ast = syn::parse_macro_input!(input as NalUnitStruct);
}
