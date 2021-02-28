use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use syn::{
  braced,
  parse::{Parse, ParseStream},
  token, Attribute, Expr, Label, Pat, Result, Token,
};

use crate::{
  block::Block,
  data::{Field, Skip},
  syn_private,
  traits::{FieldIter, FlatFields},
  ty::ComplexType,
};

pub enum Stmt {
  If(ExprIf),
  For(ExprForLoop),
  Skip(Skip),
  Field(Field),
}

impl Parse for Stmt {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(token::If) {
      return Ok(Self::If(input.parse()?));
    }

    if input.peek(token::For) {
      return Ok(Self::For(input.parse()?));
    }

    if input.peek(token::Underscore) {
      return Ok(Self::Skip(input.parse()?));
    }

    Ok(Stmt::Field(input.parse()?))
  }
}

impl FlatFields for Stmt {
  fn flat_fields(&self) -> FieldIter {
    match self {
      Self::If(expr_if) => expr_if.flat_fields(),
      Self::For(expr_for_loop) => expr_for_loop.flat_fields(),
      Self::Skip(skip) => skip.flat_fields(),
      Self::Field(field) => field.flat_fields(),
    }
  }
}

impl ToTokens for Stmt {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    match self {
      Self::Field(field) => field.to_tokens(tokens),
      Self::If(expr_if) => expr_if.to_tokens(tokens),
      Self::Skip(skip) => skip.to_tokens(tokens),
      Self::For(expr_for_loop) => expr_for_loop.to_tokens(tokens),
    }
  }
}

pub enum ElseBranch {
  Block(Block),
  If(ExprIf),
}

impl Parse for ElseBranch {
  fn parse(input: ParseStream) -> Result<Self> {
    let lookahead = input.lookahead1();

    if input.peek(Token![if]) {
      Ok(Self::If(input.parse()?))
    } else if input.peek(token::Brace) {
      Ok(Self::Block(input.parse()?))
    } else {
      Err(lookahead.error())
    }
  }
}

impl FlatFields for ElseBranch {
  fn flat_fields<'a>(&'a self) -> FieldIter {
    let inner = match self {
      Self::Block(block) => block.flat_fields(),
      Self::If(expr_if) => expr_if.flat_fields(),
    };
    Box::new(inner.map(|x| {
      let mut x = x.clone();
      x.ty = ComplexType::Option(Box::new(x.ty.clone()));
      x
    }))
  }
}

fn append_if_block(block: &Block, tokens: &mut TokenStream, mangled_names: &HashMap<Ident, Ident>) {
  block.brace_token.surround(tokens, |tokens| {
    tokens.append_all(&block.stmts);
    tokens.append_all(block.flat_fields().map(|x| {
      let ident = &x.ident;
      let mangled = mangled_names.get(ident);
      if let ComplexType::Option(_) = &x.ty {
        quote! {#mangled = #ident;}
      } else {
        quote! {#mangled = Some(#ident);}
      }
    }));
  });
}

pub struct ExprIf {
  pub attrs: Vec<Attribute>,
  pub if_token: token::If,
  pub cond: Box<syn::Expr>,
  pub then_branch: Block,
  pub else_branch: Option<(token::Else, Box<ElseBranch>)>,
  pub is_at_root: bool,
}

impl Parse for ExprIf {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(ExprIf {
      attrs: input.call(Attribute::parse_outer)?,
      if_token: input.parse()?,
      cond: Box::new(input.call(syn::Expr::parse_without_eager_brace)?),
      then_branch: input.parse()?,
      else_branch: {
        if let Some(else_token) = input.parse::<Option<token::Else>>()? {
          Some((else_token, Box::new(input.parse()?)))
        } else {
          None
        }
      },
      is_at_root: false,
    })
  }
}

impl FlatFields for ExprIf {
  fn flat_fields(&self) -> FieldIter {
    let mut iterators = vec![self.then_branch.flat_fields()];

    if let Some((_, expr)) = &self.else_branch {
      iterators.push(expr.flat_fields());
    }

    Box::new(iterators.into_iter().flatten().map(|x| {
      let mut x = x.clone();
      x.ty = ComplexType::Option(Box::new(x.ty.clone()));
      x
    }))
  }
}

impl ToTokens for ExprIf {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    #[allow(unused_mut)]
    let mut expr_if = self;

    let mut mangled_names: HashMap<Ident, Ident> = HashMap::new();
    for field in self.flat_fields() {
      let ident = field.ident;
      let mangled = format_ident!(
        "{}_{}",
        &ident,
        thread_rng()
          .sample_iter(Alphanumeric)
          .take(4)
          .map(char::from)
          .collect::<String>()
      );
      mangled_names.insert(ident, mangled);
    }

    tokens.append_all(self.flat_fields().map(|x| {
      let mangled = mangled_names.get(&x.ident);
      let ty = &x.ty;
      quote! {
        #[allow(non_snake_case)]
        let mut #mangled: #ty = None;
      }
    }));

    loop {
      let Self {
        attrs,
        if_token,
        cond,
        then_branch,
        else_branch,
        ..
      } = expr_if;

      tokens.extend(quote! {
        #(#attrs)* #if_token #cond
      });

      append_if_block(then_branch, tokens, &mangled_names);
      match else_branch {
        Some((else_token, else_block)) => {
          else_token.to_tokens(tokens);

          match else_block.as_ref() {
            ElseBranch::Block(block) => {
              append_if_block(block, tokens, &mangled_names);
              break;
            }
            ElseBranch::If(nested) => {
              expr_if = nested;
              continue;
            }
          }
        }
        None => break,
      }
    }

    tokens.append_all(self.flat_fields().map(|x| {
      let ident = &x.ident;
      let mangled = mangled_names.get(&ident);
      quote! {
        let #ident = #mangled;
      }
    }));
  }
}

pub struct ExprForLoop {
  pub attrs: Vec<Attribute>,
  pub label: Option<Label>,
  pub for_token: Token![for],
  pub pat: Pat,
  pub in_token: Token![in],
  pub expr: Box<Expr>,
  pub body: Block,
}

impl Parse for ExprForLoop {
  fn parse(input: ParseStream) -> Result<Self> {
    let outer_attrs = input.call(Attribute::parse_outer)?;
    let label: Option<Label> = input.parse()?;
    let for_token: Token![for] = input.parse()?;

    let pat = syn_private::parsing::multi_pat_with_leading_vert(input)?;

    let in_token: Token![in] = input.parse()?;
    let expr: Expr = input.call(Expr::parse_without_eager_brace)?;

    let content;
    let brace_token = braced!(content in input);
    let inner_attrs = content.call(Attribute::parse_inner)?;
    let stmts = content.call(Block::parse_within)?;

    let body = Block { brace_token, stmts };

    Ok(ExprForLoop {
      attrs: syn_private::private::attrs(outer_attrs, inner_attrs),
      label,
      for_token,
      pat,
      in_token,
      expr: Box::new(expr),
      body,
    })
  }
}

impl FlatFields for ExprForLoop {
  fn flat_fields<'a>(&'a self) -> FieldIter {
    Box::new(self.body.flat_fields().map(|x| {
      let mut x = x.clone();
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
    let mut mangled_names: HashMap<Ident, Ident> = HashMap::new();
    for field in self.flat_fields() {
      let ident = field.ident;
      let mangled = format_ident!(
        "{}_{}",
        &ident,
        thread_rng()
          .sample_iter(Alphanumeric)
          .take(4)
          .map(char::from)
          .collect::<String>()
      );
      mangled_names.insert(ident, mangled);
    }

    tokens.append_all(self.flat_fields().map(|x| {
      let mangled = mangled_names.get(&x.ident);
      let ty = &x.ty;
      quote! {
        #[allow(non_snake_case)]
        let mut #mangled: #ty = std::vec::Vec::new();
      }
    }));

    printing::outer_attrs_to_tokens(&self.attrs, tokens);
    self.label.to_tokens(tokens);
    self.for_token.to_tokens(tokens);
    self.pat.to_tokens(tokens);
    self.in_token.to_tokens(tokens);
    printing::wrap_bare_struct(tokens, &self.expr);

    self.body.brace_token.surround(tokens, |tokens| {
      printing::inner_attrs_to_tokens(&self.attrs, tokens);
      tokens.append_all(&self.body.stmts);

      tokens.append_all(self.body.flat_fields().map(|x| {
        let ident = &x.ident;
        let mangled = mangled_names.get(ident);
        quote! {#mangled.push(#ident);}
      }));
    });

    tokens.append_all(self.flat_fields().map(|x| {
      let ident = &x.ident;
      let mangled = mangled_names.get(&x.ident);
      quote! {
        let #ident = #mangled;
      }
    }));
  }
}
