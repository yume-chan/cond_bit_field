extern crate proc_macro;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::{braced,
          parse::{Parse, ParseStream},
          parse_macro_input,
          spanned::Spanned,
          token, Attribute, Error, Expr, Ident, PathArguments, Result, Token, TypePath, Visibility};

trait FlatFields {
  fn flat_fields(&self) -> Vec<&Field>;
}

struct Struct {
  pub attrs: Vec<Attribute>,
  pub vis: Visibility,
  pub struct_token: Token![struct],
  pub ident: Ident,
  pub brace_token: token::Brace,
  pub items: Vec<StructItem>,
}

impl Parse for Struct {
  fn parse(input: ParseStream) -> Result<Self> {
    let content;
    Ok(Struct {
      attrs: input.call(Attribute::parse_outer)?,
      vis: input.parse()?,
      struct_token: input.parse()?,
      ident: input.parse()?,
      brace_token: braced!(content in input),
      items: StructItem::parse(&content, false)?,
    })
  }
}

impl ToTokens for Struct {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    tokens.append_all(&self.attrs);
    self.vis.to_tokens(tokens);
    self.struct_token.to_tokens(tokens);
    self.ident.to_tokens(tokens);

    let fields = self
      .items
      .iter()
      .flat_map(|x| x.flat_fields())
      .collect::<Vec<_>>();

    self.brace_token.surround(tokens, |tokens| {
      let definitions = fields.iter().map(
        |Field {
           optional,
           attrs,
           vis,
           ident,
           colon_token,
           ty,
           ..
         }| {
          let ty = if *optional {
            quote_spanned!(ty.span()=> Option<#ty>)
          } else {
            quote! {#ty}
          };

          quote! {
            #(#attrs)* #vis #ident #colon_token #ty
          }
        },
      );

      tokens.extend(quote! {
        #(#definitions),*
      });
    });

    let ident = &self.ident;
    tokens.extend(quote! {impl BitField for #ident});
    self.brace_token.surround(tokens, |tokens| {
      let mut content = TokenStream::new();

      for field in fields.iter() {
        if field.optional {
          content.append(Ident::new("let", field.ident.span()));
          content.append(Ident::new("mut", field.ident.span()));
          field.ident.to_tokens(&mut content);
          field.colon_token.to_tokens(&mut content);

          let ty = &field.ty;
          let span = ty.span();
          content.extend(quote_spanned!(span=> Option<#ty>));
          content.extend(quote! {= None});

          field.semicolon_token.to_tokens(&mut content);
        }
      }

      for item in self.items.iter() {
        item.to_tokens(&mut content);
      }

      let field_names = fields.iter().map(|x| &x.ident);
      content.extend(quote! {
        Ok(#ident{
          #(#field_names),*
        })
      });

      tokens.extend(quote! {
        fn read(reader: &mut impl BitRead) -> Result<Self> {
          #content
        }
      })
    });
  }
}

enum StructItem {
  ExprIf(ExprIf),
  Field(Field),
}

impl StructItem {
  fn parse(input: ParseStream, optional: bool) -> Result<Vec<Self>> {
    let mut items = Vec::new();
    while !input.is_empty() {
      if input.peek(Token![if]) {
        items.push(Self::ExprIf(ExprIf::parse(input, optional)?));
      } else {
        items.push(Self::Field(Field::parse(input, optional)?));
      }
    }
    Ok(items)
  }
}

impl FlatFields for StructItem {
  fn flat_fields(&self) -> Vec<&Field> {
    match self {
      Self::ExprIf(expr_if) => expr_if.flat_fields(),
      Self::Field(field) => field.flat_fields(),
    }
  }
}

impl ToTokens for StructItem {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    match self {
      Self::ExprIf(expr_if) => expr_if.to_tokens(tokens),
      Self::Field(field) => field.to_tokens(tokens),
    }
  }
}

struct ExprIf {
  pub attrs: Vec<Attribute>,
  pub if_token: Token![if],
  pub cond: Box<Expr>,
  pub brace_token: token::Brace,
  pub then_items: Vec<StructItem>,
  // pub else_branch: Option<(Token![else], Box<Expr>)>,
}

impl ExprIf {
  fn parse(input: ParseStream, _optional: bool) -> Result<Self> {
    let then_content;
    Ok(ExprIf {
      attrs: input.call(Attribute::parse_outer)?,
      if_token: input.parse()?,
      cond: Box::new(input.call(Expr::parse_without_eager_brace)?),
      brace_token: braced!(then_content in input),
      then_items: then_content.call(|x| StructItem::parse(x, true))?,
    })
  }
}

impl FlatFields for ExprIf {
  fn flat_fields(&self) -> Vec<&Field> {
    self
      .then_items
      .iter()
      .flat_map(|x| x.flat_fields())
      .collect::<Vec<_>>()
  }
}

impl ToTokens for ExprIf {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    tokens.append_all(&self.attrs);
    self.if_token.to_tokens(tokens);
    self.cond.to_tokens(tokens);
    self.brace_token.surround(tokens, |tokens| {
      tokens.append_all(&self.then_items);
    });
  }
}

enum FieldType {
  Bool { span: Span },
  Number { signed: bool, size: u8, span: Span },
  Struct(TypePath),
}

impl Parse for FieldType {
  fn parse(input: ParseStream) -> Result<Self> {
    let ty = input.parse::<TypePath>()?;

    if ty.qself != None {
      return Ok(FieldType::Struct(ty));
    }

    let path = &ty.path;
    if path.leading_colon != None {
      return Ok(FieldType::Struct(ty));
    }

    if path.segments.len() != 1 {
      return Ok(FieldType::Struct(ty));
    }

    let segment = &path.segments[0];
    if segment.arguments != PathArguments::None {
      return Ok(FieldType::Struct(ty));
    }

    let name = segment.ident.to_string();
    if name == "bool" {
      return Ok(FieldType::Bool { span: ty.span() });
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

        Ok(FieldType::Number {
          signed,
          size,
          span: ty.span(),
        })
      }
      Err(_) => Ok(FieldType::Struct(ty)),
    }
  }
}

impl ToTokens for FieldType {
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

struct Field {
  pub optional: bool,

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
  pub ty: FieldType,

  pub semicolon_token: token::Semi,
}

impl Field {
  pub fn parse(input: ParseStream, optional: bool) -> Result<Self> {
    let attrs = input.call(Attribute::parse_outer)?;
    let vis = input.parse()?;
    let ident = match input.parse::<Option<token::Underscore>>()? {
      Some(_) => None,
      None => input.parse()?,
    };
    let colon_token = input.parse()?;
    let ty = input.parse()?;
    let semicolon_token = input.parse()?;

    if ident == None && !matches!(ty, FieldType::Number { .. }) {
      return Err(Error::new_spanned(
        ty,
        "Type of skip members must be integer",
      ));
    }

    Ok(Field {
      optional,
      attrs,
      vis,
      ident,
      colon_token,
      ty,
      semicolon_token,
    })
  }
}

impl FlatFields for Field {
  fn flat_fields(&self) -> Vec<&Field> {
    if self.ident == None {
      return vec![];
    }

    vec![self]
  }
}

impl ToTokens for Field {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    if self.ident == None {
      let span = self.ident.span();
      let size = match self.ty {
        FieldType::Number { size, .. } => size,
        _ => unreachable!(),
      };
      tokens.extend(quote_spanned! {span=> reader.skip(#size)?;});
      return;
    }

    let Field { ident, ty, .. } = self;

    if !self.optional {
      tokens.extend(quote! {let #ident: #ty =});
    } else {
      tokens.extend(quote! {#ident =});
    }

    let parser = match ty {
      FieldType::Bool { .. } => quote! {reader.read_bit()? == 1},
      FieldType::Number { size, .. } => quote! {reader.read_sized(#size)?},
      FieldType::Struct(_) => {
        quote! {reader.read()?}
      }
    };

    if self.optional {
      tokens.extend(quote! {Some(#parser)});
    } else {
      tokens.extend(parser);
    }

    self.semicolon_token.to_tokens(tokens);
  }
}

#[proc_macro]
pub fn cond_bit_field(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let data = parse_macro_input!(input as Struct);
  proc_macro::TokenStream::from(quote! {#data})
}
