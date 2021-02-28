use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::{
  braced,
  parse::{Parse, ParseStream},
  token, Result,
};

use crate::{
  stmt::Stmt,
  traits::{FieldIter, FlatFields},
};

pub struct Block {
  pub brace_token: token::Brace,
  pub stmts: Vec<Stmt>,
}

impl Block {
  pub fn parse_within(input: ParseStream) -> Result<Vec<Stmt>> {
    let mut stmts = Vec::new();
    while !input.is_empty() {
      stmts.push(input.parse()?);
    }
    Ok(stmts)
  }
}

impl Parse for Block {
  fn parse(input: ParseStream) -> Result<Self> {
    let content;
    Ok(Self {
      brace_token: braced!(content in input),
      stmts: content.call(Self::parse_within)?,
    })
  }
}

impl FlatFields for Block {
  fn flat_fields(&self) -> FieldIter {
    Box::new(self.stmts.iter().flat_map(|x| x.flat_fields()))
  }
}

impl ToTokens for Block {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    self.brace_token.surround(tokens, |tokens| {
      tokens.append_all(&self.stmts);
    });
  }
}
