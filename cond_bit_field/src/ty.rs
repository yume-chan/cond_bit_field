use core::ops::Deref;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote_spanned, ToTokens, TokenStreamExt};
use syn::{
  parse::{Parse, ParseStream},
  spanned::Spanned,
  Error, PathArguments, Result, TypePath,
};

#[derive(Clone)]
pub enum Type {
  Bool { span: Span },
  Number { signed: bool, size: u8, span: Span },
  Struct(TypePath),
}

impl Parse for Type {
  fn parse(input: ParseStream) -> Result<Self> {
    let ty = input.parse::<TypePath>()?;

    if ty.qself != None {
      return Ok(Type::Struct(ty));
    }

    let path = &ty.path;
    if path.leading_colon != None {
      return Ok(Type::Struct(ty));
    }

    if path.segments.len() != 1 {
      return Ok(Type::Struct(ty));
    }

    let segment = &path.segments[0];
    if segment.arguments != PathArguments::None {
      return Ok(Type::Struct(ty));
    }

    let name = segment.ident.to_string();
    if name == "bool" {
      return Ok(Type::Bool { span: ty.span() });
    }

    let signed = name
      .chars()
      .nth(0)
      .ok_or(Error::new(ty.span(), "invalid type name"))?
      == 'i';

    match name[1..].parse::<u8>() {
      Ok(size) => {
        if size > 128 {
          return Err(Error::new(ty.span(), "size is too large"));
        }

        Ok(Type::Number {
          signed,
          size,
          span: ty.span(),
        })
      }
      Err(_) => Ok(Type::Struct(ty)),
    }
  }
}

impl ToTokens for Type {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    match self {
      Self::Bool { span } => tokens.extend(quote_spanned!(*span=> bool)),
      Self::Number { signed, size, span } => {
        if size < &8 {
          tokens.append(Ident::new("u8", *span));
        } else {
          let mut ident = format_ident!(
            "{}{}",
            if *signed { 'i' } else { 'u' },
            (2.0f32)
              .powi((*size as f32).log2().ceil() as i32)
              .to_string()
          );
          ident.set_span(*span);
          tokens.append(ident);
        }
      }
      Self::Struct(ty) => {
        ty.to_tokens(tokens);
      }
    }
  }
}

#[derive(Clone)]
pub enum ComplexType {
  Simple(Type),
  Vec(Box<ComplexType>),
  Option(Box<ComplexType>),
}

impl ComplexType {
  pub fn collapse(&self) -> ComplexType {
    match self {
      Self::Simple(_) => self.clone(),
      Self::Vec(inner) => Self::Vec(Box::new(inner.collapse())),
      Self::Option(inner) => {
        let mut inner = inner;
        loop {
          match &**inner {
            Self::Option(inner2) => inner = inner2,
            _ => break Self::Option(Box::new(inner.collapse())),
          }
        }
      }
    }
  }
}

impl Deref for ComplexType {
  type Target = Type;

  fn deref(&self) -> &Self::Target {
    match self {
      Self::Simple(inner) => &inner,
      Self::Vec(inner) => &**inner,
      Self::Option(inner) => &**inner,
    }
  }
}

impl ToTokens for ComplexType {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    match self.collapse() {
      Self::Simple(inner) => inner.to_tokens(tokens),
      Self::Vec(inner) => tokens.extend(quote_spanned!(inner.span()=> std::vec::Vec<#inner>)),
      Self::Option(inner) => {
        tokens.extend(quote_spanned!(inner.span()=> std::option::Option<#inner>))
      }
    }
  }
}
