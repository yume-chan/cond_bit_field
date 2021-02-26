use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{braced,
          parse::{Parse, ParseStream},
          token, Attribute, Result, Token};

use crate::{data::Field,
            traits::{FieldIter, FieldIterMut, FlatFields},
            ty::{OptionType, Type}};

pub enum Stmt {
  Field(Field),
  If(ExprIf),
}

impl Parse for Stmt {
  fn parse(input: ParseStream) -> Result<Self> {
    if input.peek(token::If) {
      return Ok(Self::If(input.parse()?));
    }

    if input.peek(Token![for]) {
      todo!();
    }

    Ok(Stmt::Field(input.parse()?))
  }
}

impl FlatFields for Stmt {
  fn flat_fields(&self) -> FieldIter {
    match self {
      Self::Field(field) => field.flat_fields(),
      Self::If(expr_if) => expr_if.flat_fields(),
    }
  }

  fn flat_fields_mut(&mut self) -> FieldIterMut {
    match self {
      Self::Field(field) => field.flat_fields_mut(),
      Self::If(expr_if) => expr_if.flat_fields_mut(),
    }
  }
}

impl ToTokens for Stmt {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    match self {
      Self::Field(field) => field.to_tokens(tokens),
      Self::If(expr_if) => expr_if.to_tokens(tokens),
    }
  }
}

pub struct Block {
  pub brace_token: token::Brace,
  pub items: Vec<Stmt>,
}

impl Block {
  fn parse_within(input: ParseStream) -> Result<Vec<Stmt>> {
    let mut stmts = Vec::new();
    while !input.is_empty() {
      stmts.push(input.parse()?);
    }
    Ok(stmts)
  }
}

impl Parse for Block {
  fn parse(input: ParseStream) -> Result<Self> {
    let content;
    Ok(Block {
      brace_token: braced!(content in input),
      items: content.call(Block::parse_within)?,
    })
  }
}

impl FlatFields for Block {
  fn flat_fields<'a>(&'a self) -> FieldIter {
    Box::new(self.items.iter().flat_map(|x| x.flat_fields()))
  }

  fn flat_fields_mut<'a>(&'a mut self) -> FieldIterMut {
    Box::new(self.items.iter_mut().flat_map(|x| x.flat_fields_mut()))
  }
}

impl ToTokens for Block {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    self.brace_token.surround(tokens, |tokens| {
      tokens.append_all(&self.items);
    });
  }
}

pub enum ElseBranch {
  Block(Block),
  If(ExprIf),
}

impl Parse for ElseBranch {
  fn parse(input: ParseStream) -> Result<Self> {
    let lookahead = input.lookahead1();
    let else_branch = if input.peek(Token![if]) {
      Self::If({
        let mut expr_if: ExprIf = input.parse()?;
        expr_if.is_nested = true;
        expr_if
      })
    } else if input.peek(token::Brace) {
      let mut block: Block = input.parse()?;

      for field in block.flat_fields_mut() {
        let ty = std::mem::replace(
          &mut field.ty,
          OptionType::from_type(Type::Bool {
            span: Span::call_site(),
          }),
        );
        field.ty = match ty {
          OptionType::Simple(ty) => OptionType::Option(ty),
          OptionType::Option(ty) => OptionType::Option(ty),
        };
      }

      ElseBranch::Block(block)
    } else {
      return Err(lookahead.error());
    };

    Ok(else_branch)
  }
}

impl FlatFields for ElseBranch {
  fn flat_fields<'a>(&'a self) -> FieldIter {
    match self {
      Self::Block(block) => block.flat_fields(),
      Self::If(expr_if) => expr_if.flat_fields(),
    }
  }

  fn flat_fields_mut<'a>(&'a mut self) -> FieldIterMut {
    match self {
      Self::Block(block) => block.flat_fields_mut(),
      Self::If(expr_if) => expr_if.flat_fields_mut(),
    }
  }
}

impl ToTokens for ElseBranch {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    match self {
      Self::Block(block) => {
        let then_block = &block.items;
        let fields = block.flat_fields().map(|x| {
          let ident = x.ident.as_ref().unwrap();
          let mangled = format_ident!("_{}", ident);
          quote! {#mangled = Some(#ident);}
        });
        tokens.extend(quote! {
          {
             #(#then_block)*
             #(#fields)*
          }
        });
      }
      Self::If(expr_if) => expr_if.to_tokens(tokens),
    }
  }
}

pub struct ExprIf {
  pub attrs: Vec<Attribute>,
  pub if_token: token::If,
  pub cond: Box<syn::Expr>,
  pub then_branch: Block,
  pub else_branch: Option<(token::Else, Box<ElseBranch>)>,
  pub is_nested: bool,
}

impl Parse for ExprIf {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(ExprIf {
      attrs: input.call(Attribute::parse_outer)?,
      if_token: input.parse()?,
      cond: Box::new(input.call(syn::Expr::parse_without_eager_brace)?),
      then_branch: {
        let mut block: Block = input.parse()?;

        for field in block.flat_fields_mut() {
          let ty = std::mem::replace(
            &mut field.ty,
            OptionType::from_type(Type::Bool {
              span: Span::call_site(),
            }),
          );
          field.ty = match ty {
            OptionType::Simple(ty) => OptionType::Option(ty),
            OptionType::Option(ty) => OptionType::Option(ty),
          };
        }

        block
      },
      else_branch: {
        if let Some(else_token) = input.parse::<Option<token::Else>>()? {
          Some((else_token, Box::new(input.parse()?)))
        } else {
          None
        }
      },
      is_nested: false,
    })
  }
}

impl FlatFields for ExprIf {
  fn flat_fields<'a>(&'a self) -> FieldIter {
    let mut iterators = vec![self.then_branch.flat_fields()];

    if let Some((_, expr)) = &self.else_branch {
      iterators.push(expr.flat_fields());
    }

    Box::new(iterators.into_iter().flat_map(|x| x))
  }

  fn flat_fields_mut<'a>(&'a mut self) -> FieldIterMut {
    let mut iterators = vec![self.then_branch.flat_fields_mut()];

    if let Some((_, expr)) = &mut self.else_branch {
      iterators.push(expr.flat_fields_mut());
    }

    Box::new(iterators.into_iter().flat_map(|x| x))
  }
}

impl ToTokens for ExprIf {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    let Self {
      attrs,
      if_token,
      cond,
      then_branch,
      else_branch,
      is_nested,
    } = self;

    if !is_nested {
      let fields = self.flat_fields().map(|x| {
        let mangled = format_ident!("_{}", x.ident.as_ref().unwrap());
        let ty = &x.ty;
        quote! {let mut #mangled: #ty = None;}
      });
      tokens.extend(quote! {
        #(#fields);*
      });
    }

    let then_block = &then_branch.items;
    let fields = then_branch.flat_fields().map(|x| {
      let ident = x.ident.as_ref().unwrap();
      let mangled = format_ident!("_{}", ident);
      quote! {#mangled = Some(#ident);}
    });
    tokens.extend(quote! {
      #(#attrs)* #if_token #cond {
        #(#then_block)*
        #(#fields)*
      }
    });

    if let Some((else_token, else_block)) = else_branch {
      tokens.extend(quote! {
        #else_token #else_block
      });
    }

    if !is_nested {
      let fields = self.flat_fields().map(|x| {
        let ident = x.ident.as_ref().unwrap();
        let mangled = format_ident!("_{}", x.ident.as_ref().unwrap());
        quote! {let #ident = #mangled;}
      });
      tokens.extend(quote! {
        #(#fields);*
      });
    }
  }
}
