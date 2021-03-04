use std::iter;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parenthesized,
          parse::{Parse, ParseStream},
          parse2, token, Attribute, LitInt, Result, Token, Visibility};

use crate::{block::ExprBlock,
            traits::{FieldIter, FlatFields},
            ty::{ComplexType, Type}};

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

pub struct SizeAttributeArgs {
  expr: syn::Expr,
}

impl Parse for SizeAttributeArgs {
  fn parse(input: ParseStream) -> Result<Self> {
    let content;
    parenthesized!(content in input);
    Ok(Self {
      expr: content.call(syn::Expr::parse_without_eager_brace)?,
    })
  }
}

#[derive(Clone)]
pub struct Field {
  /// Attributes tagged on the field.
  pub attrs: Vec<Attribute>,

  pub size: Option<syn::Expr>,

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

impl Field {
  fn to_initializer(&self) -> TokenStream {
    match self.size {
      Some(ref size) => quote! {reader.read_sized(#size)?},
      None => match *self.ty {
        Type::Bool { .. } => quote! {reader.read_bit()?},
        Type::Number { size, .. } => quote! {reader.read_sized(#size)?},
        Type::Struct(_) => {
          quote! {reader.read()?}
        }
      },
    }
  }
}

impl Parse for Field {
  fn parse(input: ParseStream) -> Result<Self> {
    let attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;
    let mut size_attribute: Option<Attribute> = None;
    let attrs = attrs
      .into_iter()
      .filter(|x| {
        let segments = &x.path.segments;
        if segments.len() == 1 {
          let first = segments.first().unwrap();
          if first.ident.to_string() == "size" {
            size_attribute = Some(x.clone());
            return false;
          }
        }
        true
      })
      .collect::<Vec<_>>();

    let size = match size_attribute {
      Some(attribute) => {
        let args = parse2::<SizeAttributeArgs>(attribute.tokens)?;
        Some(args.expr)
      }
      None => None,
    };

    Ok(Field {
      attrs: attrs,
      size,
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

    let ty = &**ty;
    let initializer = self.to_initializer();
    tokens.extend(quote! {let #ident: #ty = #initializer #semicolon_token});
  }
}

pub struct Struct {
  pub attrs: Vec<Attribute>,
  pub vis: Visibility,
  pub struct_token: Token![struct],
  pub ident: Ident,
  pub fields: ExprBlock,
}

impl Parse for Struct {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(Struct {
      attrs: input.call(Attribute::parse_outer)?,
      vis: input.parse()?,
      struct_token: input.parse()?,
      ident: input.parse()?,
      fields: input.parse()?,
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
      impl cond_bit_stream::BitField for #ident {
        fn read(reader: &mut impl cond_bit_stream::BitRead) -> cond_bit_stream::Result<Self> {
          #(#initializers)*
          Ok(#ident {
            #(#field_names),*
          })
        }
      }
    });
  }
}
