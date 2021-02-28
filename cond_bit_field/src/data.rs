use std::iter;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
  parse::{Parse, ParseStream},
  token, Attribute, LitInt, Result, Token, Visibility,
};

use crate::{
  block::Block,
  stmt::Stmt,
  traits::{FieldIter, FlatFields},
  ty::{ComplexType, Type},
};

pub struct Skip {
  pub underscore_token: token::Underscore,

  pub colon_token: token::Colon,

  pub size: LitInt,

  pub semicolon_token: token::Semi,
}

impl Parse for Skip {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(Skip {
      underscore_token: input.parse()?,
      colon_token: input.parse()?,
      size: input.parse()?,
      semicolon_token: input.parse()?,
    })
  }
}

impl FlatFields for Skip {
  fn flat_fields(&self) -> FieldIter {
    Box::new(iter::empty())
  }
}

impl ToTokens for Skip {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let size = &self.size;
    tokens.extend(quote! {reader.skip(#size)?;});
  }
}

#[derive(Clone)]
pub struct Field {
  /// Attributes tagged on the field.
  pub attrs: Vec<Attribute>,

  /// Visibility of the field.
  pub vis: Visibility,

  /// Name of the field, if any.
  ///
  /// Fields of tuple structs have no names.
  pub ident: Ident,

  pub colon_token: token::Colon,

  /// Type of the field.
  pub ty: ComplexType,

  pub semicolon_token: token::Semi,
}

impl Parse for Field {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(Field {
      attrs: input.call(Attribute::parse_outer)?,
      vis: input.parse()?,
      ident: input.parse()?,
      colon_token: input.parse()?,
      ty: ComplexType::Simple(input.parse()?),
      semicolon_token: input.parse()?,
    })
  }
}

impl FlatFields for Field {
  fn flat_fields(&self) -> FieldIter {
    Box::new(iter::once(self.clone()))
  }
}

impl ToTokens for Field {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let Field {
      ident,
      ty,
      semicolon_token,
      ..
    } = self;

    let parser = match **ty {
      Type::Bool { .. } => quote! {reader.read_bit()? == 1},
      Type::Number { size, .. } => quote! {reader.read_sized(#size)?},
      Type::Struct(_) => {
        quote! {reader.read()?}
      }
    };

    let ty = &**ty;
    tokens.extend(quote! {let #ident: #ty = #parser #semicolon_token});
  }
}

pub struct Struct {
  pub attrs: Vec<Attribute>,
  pub vis: Visibility,
  pub struct_token: Token![struct],
  pub ident: Ident,
  pub fields: Block,
}

impl Parse for Struct {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(Struct {
      attrs: input.call(Attribute::parse_outer)?,
      vis: input.parse()?,
      struct_token: input.parse()?,
      ident: input.parse()?,
      fields: {
        let mut fields: Block = input.parse()?;
        for stmt in fields.stmts.iter_mut() {
          if let Stmt::If(expr_if) = stmt {
            expr_if.is_at_root = true;
          }
        }
        fields
      },
    })
  }
}

impl ToTokens for Struct {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let Self {
      attrs,
      vis,
      struct_token,
      ident,
      fields,
    } = self;

    tokens.extend(quote! {
      #(#attrs)* #vis #struct_token #ident
    });

    fields.brace_token.surround(tokens, |tokens| {
      tokens.append_all(fields.flat_fields().map(
        |Field {
           attrs,
           vis,
           ident,
           colon_token,
           ty,
           ..
         }| {
          quote! {
            #(#attrs)* #vis #ident #colon_token #ty,
          }
        },
      ));
    });

    let initializers = &fields.stmts;
    let field_names = fields.flat_fields().map(|x| x.ident);
    tokens.extend(quote! {
      impl BitField for #ident {
        fn read(reader: &mut impl BitRead) -> Result<Self> {
          #(#initializers)*
          Ok(#ident {
            #(#field_names),*
          })
        }
      }
    });
  }
}
