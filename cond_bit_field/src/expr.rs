use std::iter;

use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::{braced,
          parse::{Parse, ParseStream},
          token, Attribute, Label, Pat, Result, Token};

use crate::{block::{ExprBlock, Unshadow},
            data::{Field, Skip},
            r#if::ExprIf,
            r#match::ExprMatch,
            syn_private,
            traits::{FieldIter, FlatFields},
            ty::ComplexType};

pub enum Expr {
    Block(ExprBlock),
    Field(Field),
    ForLoop(ExprForLoop),
    If(ExprIf),
    Local(syn::Local),
    Match(ExprMatch),
    Skip(Skip),
}

impl Expr {
    pub fn replace_attrs(&mut self, new: Vec<Attribute>) -> Vec<Attribute> {
        match self {
            Expr::Local(syn::Local { attrs, .. })
            | Expr::If(ExprIf { attrs, .. })
            | Expr::ForLoop(ExprForLoop { attrs, .. })
            | Expr::Match(ExprMatch { attrs, .. })
            | Expr::Block(ExprBlock { attrs, .. })
            | Expr::Field(Field { attrs, .. }) => std::mem::replace(attrs, new),
            Self::Skip(_) => Vec::new(),
        }
    }

    pub fn unary_expr(input: ParseStream) -> Result<Self> {
        if input.peek(token::Brace) {
            return Ok(Self::Block(input.parse()?));
        }

        if input.peek(token::For) {
            return Ok(Self::ForLoop(input.parse()?));
        }

        if input.peek(token::If) {
            return Ok(Self::If(input.parse()?));
        }

        if input.peek(token::Let) {
            let local = match syn::Stmt::parse(input)? {
                syn::Stmt::Local(local) => local,
                _ => unreachable!(),
            };
            return Ok(Self::Local(local));
        }

        if input.peek(token::Match) {
            return Ok(Self::Match(input.parse()?));
        }

        if input.peek(token::Underscore) {
            return Ok(Self::Skip(input.parse()?));
        }

        Ok(Expr::Field(input.parse()?))
    }
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut attrs = input.call(Attribute::parse_outer)?;
        let mut atom = Self::unary_expr(input)?;
        attrs.extend(atom.replace_attrs(Vec::new()));
        atom.replace_attrs(attrs);
        Ok(atom)
    }
}

impl FlatFields for Expr {
    fn flat_fields(&self) -> FieldIter {
        match self {
            Self::Block(block) => block.flat_fields(),
            Self::Field(field) => field.flat_fields(),
            Self::ForLoop(expr_for_loop) => expr_for_loop.flat_fields(),
            Self::If(expr_if) => expr_if.flat_fields(),
            Self::Local(..) => Box::new(iter::empty()),
            Self::Match(expr_match) => expr_match.flat_fields(),
            Self::Skip(skip) => skip.flat_fields(),
        }
    }
}

impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Block(block) => block.to_unshadow(tokens),
            Self::Field(field) => field.to_tokens(tokens),
            Self::ForLoop(expr_for_loop) => expr_for_loop.to_tokens(tokens),
            Self::If(expr_if) => expr_if.to_tokens(tokens),
            Self::Local(expr_let) => expr_let.to_tokens(tokens),
            Self::Match(expr_match) => expr_match.to_tokens(tokens),
            Self::Skip(skip) => skip.to_tokens(tokens),
        }
    }
}

pub struct ExprForLoop {
    pub attrs: Vec<Attribute>,
    pub label: Option<Label>,
    pub for_token: Token![for],
    pub pat: Pat,
    pub in_token: Token![in],
    pub expr: syn::Expr,
    pub body: ExprBlock,
}

impl Parse for ExprForLoop {
    fn parse(input: ParseStream) -> Result<Self> {
        let outer_attrs = input.call(Attribute::parse_outer)?;
        let label: Option<Label> = input.parse()?;
        let for_token: Token![for] = input.parse()?;

        let pat = syn_private::pat::parsing::multi_pat_with_leading_vert(input)?;

        let in_token: Token![in] = input.parse()?;
        let expr: syn::Expr = input.call(syn::Expr::parse_without_eager_brace)?;

        let content;
        let brace_token = braced!(content in input);
        let inner_attrs = content.call(Attribute::parse_inner)?;
        let stmts = content.call(ExprBlock::parse_within)?;

        let body = ExprBlock {
            attrs: Vec::new(),
            brace_token,
            stmts,
        };

        Ok(ExprForLoop {
            attrs: syn_private::private::attrs(outer_attrs, inner_attrs),
            label,
            for_token,
            pat,
            in_token,
            expr,
            body,
        })
    }
}

impl FlatFields for ExprForLoop {
    fn flat_fields<'a>(&'a self) -> FieldIter {
        Box::new(self.body.flat_fields().map(|x| {
            let mut x = x;
            x.ty = ComplexType::Vec(Box::new(x.ty.clone()));
            x
        }))
    }
}

impl ToTokens for ExprForLoop {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let unshadow = Unshadow::new(self);
        unshadow.to_dec(tokens);

        syn_private::printing::outer_attrs_to_tokens(&self.attrs, tokens);
        self.label.to_tokens(tokens);
        self.for_token.to_tokens(tokens);
        self.pat.to_tokens(tokens);
        self.in_token.to_tokens(tokens);
        syn_private::printing::wrap_bare_struct(tokens, &self.expr);

        self.body.brace_token.surround(tokens, |tokens| {
            syn_private::printing::inner_attrs_to_tokens(&self.attrs, tokens);
            tokens.append_all(&self.body.stmts);
            unshadow.to_backup(&self.body, tokens);
        });

        unshadow.to_restore(tokens);
    }
}
