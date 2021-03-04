use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::{braced,
          parse::{Parse, ParseStream},
          token, Attribute, Label, Pat, Result, Token};

use crate::{block::{ExprBlock, Unshadow},
            data::{Field, Skip},
            r#if::ExprIf,
            r#match::ExprMatch,
            syn_private,
            traits::{FieldIter, FlatFields},
            ty::ComplexType};

pub enum Expr {
  Block(ExprBlock),
  Field(Field),
  ForLoop(ExprForLoop),
  If(ExprIf),
  Match(ExprMatch),
  Skip(Skip),
}

impl Parse for Expr {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(token::Brace) {
      return Ok(Self::Block(input.parse()?));
    }

    if input.peek(token::For) {
      return Ok(Self::ForLoop(input.parse()?));
    }

    if input.peek(token::If) {
      return Ok(Self::If(input.parse()?));
    }

    if input.peek(token::Match) {
      return Ok(Self::Match(input.parse()?));
    }

    if input.peek(token::Underscore) {
      return Ok(Self::Skip(input.parse()?));
    }

    Ok(Expr::Field(input.parse()?))
  }
}

impl FlatFields for Expr {
  fn flat_fields(&self) -> FieldIter {
    match self {
      Self::Block(block) => block.flat_fields(),
      Self::Field(field) => field.flat_fields(),
      Self::ForLoop(expr_for_loop) => expr_for_loop.flat_fields(),
      Self::If(expr_if) => expr_if.flat_fields(),
      Self::Match(expr_match) => expr_match.flat_fields(),
      Self::Skip(skip) => skip.flat_fields(),
    }
  }
}

impl ToTokens for Expr {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    match self {
      Self::Block(block) => block.to_unshadow(tokens),
      Self::Field(field) => field.to_tokens(tokens),
      Self::ForLoop(expr_for_loop) => expr_for_loop.to_tokens(tokens),
      Self::If(expr_if) => expr_if.to_tokens(tokens),
      Self::Match(expr_match) => expr_match.to_tokens(tokens),
      Self::Skip(skip) => skip.to_tokens(tokens),
    }
  }
}

pub struct ExprForLoop {
  pub attrs: Vec<Attribute>,
  pub label: Option<Label>,
  pub for_token: Token![for],
  pub pat: Pat,
  pub in_token: Token![in],
  pub expr: syn::Expr,
  pub body: ExprBlock,
}

impl Parse for ExprForLoop {
  fn parse(input: ParseStream) -> Result<Self> {
    let outer_attrs = input.call(Attribute::parse_outer)?;
    let label: Option<Label> = input.parse()?;
    let for_token: Token![for] = input.parse()?;

    let pat = syn_private::pat::parsing::multi_pat_with_leading_vert(input)?;

    let in_token: Token![in] = input.parse()?;
    let expr: syn::Expr = input.call(syn::Expr::parse_without_eager_brace)?;

    let content;
    let brace_token = braced!(content in input);
    let inner_attrs = content.call(Attribute::parse_inner)?;
    let stmts = content.call(ExprBlock::parse_within)?;

    let body = ExprBlock { brace_token, stmts };

    Ok(ExprForLoop {
      attrs: syn_private::private::attrs(outer_attrs, inner_attrs),
      label,
      for_token,
      pat,
      in_token,
      expr,
      body,
    })
  }
}

impl FlatFields for ExprForLoop {
  fn flat_fields<'a>(&'a self) -> FieldIter {
    Box::new(self.body.flat_fields().map(|x| {
      let mut x = x;
      x.ty = ComplexType::Vec(Box::new(x.ty.clone()));
      x
    }))
  }
}

mod attr {
  use std::iter;

  use syn::{AttrStyle, Attribute};

  pub trait FilterAttrs<'a> {
    type Ret: Iterator<Item = &'a Attribute>;

    fn outer(self) -> Self::Ret;
    fn inner(self) -> Self::Ret;
  }

  impl<'a, T> FilterAttrs<'a> for T
  where
    T: IntoIterator<Item = &'a Attribute>,
  {
    type Ret = iter::Filter<T::IntoIter, fn(&&Attribute) -> bool>;

    fn outer(self) -> Self::Ret {
      fn is_outer(attr: &&Attribute) -> bool {
        match attr.style {
          AttrStyle::Outer => true,
          AttrStyle::Inner(_) => false,
        }
      }
      self.into_iter().filter(is_outer)
    }

    fn inner(self) -> Self::Ret {
      fn is_inner(attr: &&Attribute) -> bool {
        match attr.style {
          AttrStyle::Inner(_) => true,
          AttrStyle::Outer => false,
        }
      }
      self.into_iter().filter(is_inner)
    }
  }
}

pub mod printing {
  use proc_macro2::TokenStream;
  use quote::{ToTokens, TokenStreamExt};
  use syn::{token, Attribute, Expr};

  use super::attr::FilterAttrs;

  pub fn outer_attrs_to_tokens(attrs: &[Attribute], tokens: &mut TokenStream) {
    tokens.append_all(attrs.outer());
  }

  pub fn inner_attrs_to_tokens(attrs: &[Attribute], tokens: &mut TokenStream) {
    tokens.append_all(attrs.inner());
  }

  pub fn wrap_bare_struct(tokens: &mut TokenStream, e: &Expr) {
    if let Expr::Struct(_) = *e {
      token::Paren::default().surround(tokens, |tokens| {
        e.to_tokens(tokens);
      });
    } else {
      e.to_tokens(tokens);
    }
  }
}

impl ToTokens for ExprForLoop {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let unshadow = Unshadow::new(self);
    unshadow.to_dec(tokens);

    printing::outer_attrs_to_tokens(&self.attrs, tokens);
    self.label.to_tokens(tokens);
    self.for_token.to_tokens(tokens);
    self.pat.to_tokens(tokens);
    self.in_token.to_tokens(tokens);
    printing::wrap_bare_struct(tokens, &self.expr);

    self.body.brace_token.surround(tokens, |tokens| {
      printing::inner_attrs_to_tokens(&self.attrs, tokens);
      tokens.append_all(&self.body.stmts);
      unshadow.to_backup(&self.body, tokens);
    });

    unshadow.to_restore(tokens);
  }
}
