use proc_macro2::{Ident, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse::{Parse, ParseStream},
          spanned::Spanned,
          token, Attribute, Error, Result, Token, Visibility};

use crate::{stmt::Block,
            traits::{FieldIter, FieldIterMut, FlatFields},
            ty::{OptionType, Type}};

pub struct Field {
  /// Attributes tagged on the field.
  pub attrs: Vec<Attribute>,

  /// Visibility of the field.
  pub vis: Visibility,

  /// Name of the field, if any.
  ///
  /// Fields of tuple structs have no names.
  pub ident: Option<Ident>,

  pub colon_token: token::Colon,

  /// Type of the field.
  pub ty: OptionType,

  pub semicolon_token: token::Semi,
}

impl Parse for Field {
  fn parse(input: ParseStream) -> Result<Self> {
    let attrs = input.call(Attribute::parse_outer)?;
    let vis = input.parse()?;
    let ident = match input.parse::<Option<token::Underscore>>()? {
      Some(_) => None,
      None => input.parse()?,
    };
    let colon_token = input.parse()?;
    let ty = input.parse()?;
    let semicolon_token = input.parse()?;

    if ident == None && !matches!(ty, Type::Number { .. }) {
      return Err(Error::new_spanned(
        ty,
        "Type of skip members must be integer",
      ));
    }

    Ok(Field {
      attrs,
      vis,
      ident,
      colon_token,
      ty: OptionType::from_type(ty),
      semicolon_token,
    })
  }
}

impl FlatFields for Field {
  fn flat_fields(&self) -> FieldIter {
    if self.ident == None {
      Box::new((&[] as &[Self]).iter())
    } else {
      Box::new(std::slice::from_ref(self).iter())
    }
  }

  fn flat_fields_mut(&mut self) -> FieldIterMut {
    if self.ident == None {
      Box::new((&mut [] as &mut [Self]).iter_mut())
    } else {
      Box::new(std::slice::from_mut(self).iter_mut())
    }
  }
}

impl ToTokens for Field {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    if self.ident == None {
      let span = self.ident.span();
      let size = match *self.ty {
        Type::Number { size, .. } => size,
        _ => unreachable!(),
      };
      tokens.extend(quote_spanned! {span=> reader.skip(#size)?;});
      return;
    }

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

    self.fields.brace_token.surround(tokens, |tokens| {
      let definitions = self.fields.flat_fields().map(
        |Field {
           attrs,
           vis,
           ident,
           colon_token,
           ty,
           ..
         }| {
          quote! {
            #(#attrs)* #vis #ident #colon_token #ty
          }
        },
      );

      tokens.extend(quote! {
        #(#definitions),*
      });
    });

    let items = &fields.items;
    let fields = fields.flat_fields().map(|x| &x.ident);
    tokens.extend(quote! {
      impl BitField for #ident {
        fn read(reader: &mut impl BitRead) -> Result<Self> {
          #(#items)*
          Ok(#ident {
            #(#fields),*
          })
        }
      }
    });
  }
}
