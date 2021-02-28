use crate::syn_private;
use std::boxed::Box;
use syn::braced;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::token;
use syn::Attribute;
use syn::Expr;
use syn::Pat;
use syn::Result;
use syn::Token;

struct Arm {
  pub attrs: Vec<Attribute>,
  pub pat: Pat,
  pub guard: Option<(token::If, Box<Expr>)>,
  pub fat_arrow_token: token::FatArrow,
  pub body: Box<Expr>,
  pub comma: Option<token::Comma>,
}

impl Parse for Arm {
  fn parse(input: ParseStream) -> Result<Self> {
    Ok(Self {
      attrs: input.call(Attribute::parse_outer)?,
      pat: syn_private::parsing::multi_pat_with_leading_vert(input)?,
      guard: None,
      fat_arrow_token: input.parse()?,
      body: Box::new(input.parse()?),
      comma: input.parse()?,
    })
  }
}

struct ExprMatch {
  pub attrs: Vec<Attribute>,
  pub match_token: Token![match],
  pub expr: Box<Expr>,
  pub brace_token: token::Brace,
  pub arms: Vec<Arm>,
}

impl Parse for ExprMatch {
  fn parse(input: ParseStream) -> Result<Self> {
    let outer_attrs = input.call(Attribute::parse_outer)?;
    let match_token: Token![match] = input.parse()?;
    let expr = Expr::parse_without_eager_brace(input)?;

    let content;
    let brace_token = braced!(content in input);
    let inner_attrs = content.call(Attribute::parse_inner)?;

    let mut arms = Vec::new();
    while !content.is_empty() {
      arms.push(content.call(Arm::parse)?);
    }

    Ok(ExprMatch {
      attrs: syn_private::private::attrs(outer_attrs, inner_attrs),
      match_token,
      expr: Box::new(expr),
      brace_token,
      arms,
    })
  }
}
