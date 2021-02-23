extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned, ToTokens, TokenStreamExt};
use syn::{
  braced,
  parse::{Parse, ParseStream},
  parse_macro_input,
  spanned::Spanned,
  token, Attribute, Expr, Ident, PathArguments, Result, Token, TypePath, Visibility,
};

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
      for field in fields.iter() {
        tokens.append_all(&field.attrs);
        field.vis.to_tokens(tokens);
        field.ident.to_tokens(tokens);
        field.colon_token.to_tokens(tokens);

        if field.optional {
          let ty = map_type(&field.ty);
          let span = field.ty.span();
          tokens.extend(quote_spanned!(span=> Option<#ty>))
        } else {
          tokens.extend(map_type(&field.ty));
        }

        let span = field.semicolon_token.span;
        tokens.extend(quote_spanned!(span=> ,));
      }
    });

    tokens.append(Ident::new("impl", self.ident.span()));
    self.ident.to_tokens(tokens);
    self.brace_token.surround(tokens, |tokens| {
      let mut content = TokenStream::new();

      for field in fields.iter() {
        if field.optional {
          content.append(Ident::new("let", field.ident.span()));
          content.append(Ident::new("mut", field.ident.span()));
          field.ident.to_tokens(&mut content);
          field.colon_token.to_tokens(&mut content);

          let ty = map_type(&field.ty);
          let span = field.ty.span();
          content.extend(quote_spanned!(span=> Option<#ty>));
          content.extend(quote! {= None});

          field.semicolon_token.to_tokens(&mut content);
        }
      }

      for item in self.items.iter() {
        item.to_tokens(&mut content);
      }

      self.ident.to_tokens(&mut content);

      let field_names = fields.iter().map(|x| &x.ident);
      content.extend(quote! {
        {
          #(#field_names),*
        }
      });

      tokens.extend(TokenStream::from(quote! {
        pub fn new(stream: &[u8]) -> Self {
          #content
        }
      }))
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

struct Field {
  pub optional: bool,

  /// Attributes tagged on the field.
  pub attrs: Vec<Attribute>,

  /// Visibility of the field.
  pub vis: Visibility,

  /// Name of the field, if any.
  ///
  /// Fields of tuple structs have no names.
  pub ident: Ident,

  pub colon_token: Token![:],

  /// Type of the field.
  pub ty: TypePath,

  pub semicolon_token: Token![;],
}

impl Field {
  fn parse(input: ParseStream, optional: bool) -> Result<Self> {
    Ok(Field {
      optional,
      attrs: input.call(Attribute::parse_outer)?,
      vis: input.parse()?,
      ident: input.parse()?,
      colon_token: input.parse()?,
      ty: input.parse()?,
      semicolon_token: input.parse()?,
    })
  }
}

fn map_type(ty: &TypePath) -> TokenStream {
  if ty.qself != None {
    return ty.to_token_stream();
  }

  let path = &ty.path;
  if path.leading_colon != None {
    return ty.to_token_stream();
  }

  if path.segments.len() != 1 {
    return ty.to_token_stream();
  }

  let segment = &path.segments[0];
  if segment.arguments != PathArguments::None {
    return ty.to_token_stream();
  }

  let name = segment.ident.to_string();
  let span = ty.span();
  let signed = name.chars().nth(0).unwrap();
  let size = name[1..].parse::<u8>().unwrap();
  let result;

  if size <= 8 {
    result = format_ident!("{}8", signed);
  } else if size <= 16 {
    result = format_ident!("{}16", signed);
  } else if size <= 32 {
    result = format_ident!("{}32", signed);
  } else if size <= 64 {
    result = format_ident!("{}64", signed);
  } else {
    panic!();
  }

  return TokenStream::from(quote_spanned!(span=> #result));
}

impl FlatFields for Field {
  fn flat_fields(&self) -> Vec<&Field> {
    vec![self]
  }
}

impl ToTokens for Field {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    if !self.optional {
      tokens.append(Ident::new("let", self.ident.span()));
    }

    self.ident.to_tokens(tokens);

    let span = self.ty.span();
    if self.optional {
      tokens.extend(quote_spanned!(span=> = Some(0)));
    } else {
      tokens.extend(quote_spanned!(span=> = 0));
    }

    self.semicolon_token.to_tokens(tokens);
  }
}

#[proc_macro]
pub fn cond_struct(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let data = parse_macro_input!(input as Struct);
  proc_macro::TokenStream::from(quote! {#data})
}
