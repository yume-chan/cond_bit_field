use core::ops::Deref;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote_spanned, ToTokens, TokenStreamExt};
use syn::{parse::{Parse, ParseStream},
          spanned::Spanned,
          Error, PathArguments, Result, TypePath};

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

pub enum VecType {
  Simple(Type),
  Vec(Box<OptionType>),
}

impl Deref for VecType {
  type Target = Type;

  fn deref(&self) -> &Self::Target {
    match self {
      Self::Simple(ty) => &ty,
      Self::Vec(item) => &item,
    }
  }
}

impl ToTokens for VecType {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    match self {
      Self::Simple(ty) => ty.to_tokens(tokens),
      Self::Vec(item) => tokens.extend(quote_spanned!(item.span()=> std::vec::Vec<#item>)),
    }
  }
}

pub enum OptionType {
  Simple(VecType),
  Option(VecType),
}

impl OptionType {
  pub fn from_type(ty: Type) -> Self {
    Self::Simple(VecType::Simple(ty))
  }
}

impl Deref for OptionType {
  type Target = Type;

  fn deref(&self) -> &Self::Target {
    match self {
      Self::Simple(ty) => &ty,
      Self::Option(option) => &option,
    }
  }
}

impl ToTokens for OptionType {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    match self {
      Self::Simple(ty) => ty.to_tokens(tokens),
      Self::Option(item) => tokens.extend(quote_spanned!(item.span()=> std::option::Option<#item>)),
    }
  }
}
