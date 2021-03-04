use std::boxed::Box;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{braced,
          parse::{Parse, ParseStream},
          token, Attribute, Pat, Result, Token};

use crate::{block::Unshadow,
            expr::{printing::{inner_attrs_to_tokens, outer_attrs_to_tokens, wrap_bare_struct},
                   Expr},
            syn_private,
            traits::{FieldIter, FlatFields},
            ty::ComplexType};

pub struct Arm {
  pub attrs: Vec<Attribute>,
  pub pat: Pat,
  pub guard: Option<(token::If, Box<syn::Expr>)>,
  pub fat_arrow_token: token::FatArrow,
  pub body: Box<Expr>,
  pub comma: Option<token::Comma>,
}

impl Arm {
  pub fn to_backup(&self, tokens: &mut TokenStream, unshadow: &Unshadow) {
    tokens.append_all(&self.attrs);
    self.pat.to_tokens(tokens);
    if let Some((if_token, guard)) = &self.guard {
      if_token.to_tokens(tokens);
      guard.to_tokens(tokens);
    }
    self.fat_arrow_token.to_tokens(tokens);
    match &*self.body {
      Expr::Block(block) => block.to_backup(tokens, unshadow),
      _ => token::Brace::default().surround(tokens, |tokens| {
        self.body.to_tokens(tokens);
        unshadow.to_backup(self, tokens);
      }),
    }
    self.comma.to_tokens(tokens);
  }
}

// https://docs.rs/syn/1.0.60/src/syn/expr.rs.html#1049
pub fn requires_terminator(expr: &Expr) -> bool {
  // see https://github.com/rust-lang/rust/blob/2679c38fc/src/librustc_ast/util/classify.rs#L7-L25
  match expr {
    Expr::Block(..) | Expr::If(..) | Expr::Match(..) | Expr::ForLoop(..) | Expr::Field(..) => false,
    _ => true,
  }
}

impl Parse for Arm {
  fn parse(input: ParseStream) -> Result<Self> {
    let requires_comma;
    Ok(Arm {
      attrs: input.call(Attribute::parse_outer)?,
      pat: syn_private::pat::parsing::multi_pat_with_leading_vert(input)?,
      guard: {
        if input.peek(Token![if]) {
          let if_token: Token![if] = input.parse()?;
          let guard: syn::Expr = input.parse()?;
          Some((if_token, Box::new(guard)))
        } else {
          None
        }
      },
      fat_arrow_token: input.parse()?,
      body: {
        let body: Expr = input.parse()?;
        requires_comma = requires_terminator(&body);
        Box::new(body)
      },
      comma: {
        if requires_comma && !input.is_empty() {
          Some(input.parse()?)
        } else {
          input.parse()?
        }
      },
    })
  }
}

impl FlatFields for Arm {
  fn flat_fields(&self) -> FieldIter {
    Box::new(self.body.flat_fields().map(|x| {
      let mut x = x;
      x.ty = ComplexType::Option(Box::new(x.ty.clone()));
      x
    }))
  }
}

pub struct ExprMatch {
  pub attrs: Vec<Attribute>,
  pub match_token: Token![match],
  pub expr: Box<syn::Expr>,
  pub brace_token: token::Brace,
  pub arms: Vec<Arm>,
}

impl Parse for ExprMatch {
  fn parse(input: ParseStream) -> Result<Self> {
    let outer_attrs = input.call(Attribute::parse_outer)?;
    let match_token: Token![match] = input.parse()?;
    let expr = syn::Expr::parse_without_eager_brace(input)?;

    let content;
    let brace_token = braced!(content in input);
    let inner_attrs = content.call(Attribute::parse_inner)?;

    let mut arms = Vec::new();
    while !content.is_empty() {
      arms.push(content.call(Arm::parse)?);
    }

    Ok(ExprMatch {
      attrs: syn_private::private::attrs(outer_attrs, inner_attrs),
      match_token,
      expr: Box::new(expr),
      brace_token,
      arms,
    })
  }
}

impl FlatFields for ExprMatch {
  fn flat_fields(&self) -> FieldIter {
    Box::new(self.arms.iter().flat_map(|x| x.flat_fields()))
  }
}

impl ToTokens for ExprMatch {
  fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
    let unshadow = Unshadow::new(self);
    unshadow.to_dec(tokens);

    outer_attrs_to_tokens(&self.attrs, tokens);
    self.match_token.to_tokens(tokens);
    wrap_bare_struct(tokens, &self.expr);
    self.brace_token.surround(tokens, |tokens| {
      inner_attrs_to_tokens(&self.attrs, tokens);
      for (i, arm) in self.arms.iter().enumerate() {
        arm.to_backup(tokens, &unshadow);
        // Ensure that we have a comma after a non-block arm, except
        // for the last one.
        let is_last = i == self.arms.len() - 1;
        if !is_last && requires_terminator(&arm.body) && arm.comma.is_none() {
          <Token![,]>::default().to_tokens(tokens);
        }
      }
    });
    tokens.extend(quote! {;});

    unshadow.to_restore(tokens);
  }
}
