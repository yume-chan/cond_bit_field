use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::{Parse, ParseStream},
          token, Attribute, Result, Token};

use crate::{block::{ExprBlock, Unshadow},
            traits::{FieldIter, FlatFields},
            ty::ComplexType};

pub enum ElseIf {
    Block(ExprBlock),
    If(ExprIf),
}

impl Parse for ElseIf {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![if]) {
            Ok(Self::If(input.parse()?))
        } else if lookahead.peek(token::Brace) {
            Ok(Self::Block(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

impl FlatFields for ElseIf {
    fn flat_fields<'a>(&'a self) -> FieldIter {
        // All `if`s in `if {} else if {} else if {}` are treated as in the same level
        // So don't wrap another `ComplexType::Option` around them
        match self {
            Self::Block(block) => Box::new(block.flat_fields().map(|mut x| {
                x.ty = ComplexType::Option(Box::new(x.ty.clone()));
                x
            })),
            Self::If(expr_if) => expr_if.flat_fields(),
        }
    }
}

pub struct ExprIf {
    pub attrs: Vec<Attribute>,
    pub if_token: token::If,
    pub cond: Box<syn::Expr>,
    pub then_branch: ExprBlock,
    pub else_branch: Option<(token::Else, Box<ElseIf>)>,
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
        })
    }
}

impl FlatFields for ExprIf {
    fn flat_fields(&self) -> FieldIter {
        let mut iterators: Vec<FieldIter> =
            vec![Box::new(self.then_branch.flat_fields().map(|mut x| {
                x.ty = ComplexType::Option(Box::new(x.ty.clone()));
                x
            }))];

        if let Some((_, expr)) = &self.else_branch {
            iterators.push(expr.flat_fields());
        }

        Box::new(iterators.into_iter().flatten())
    }
}

impl ToTokens for ExprIf {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        #[allow(unused_mut)]
        let mut expr_if = self;

        let unshadow = Unshadow::new(self);
        unshadow.to_dec(tokens);

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

            then_branch.to_backup(tokens, &unshadow);

            match else_branch {
                Some((else_token, else_block)) => {
                    else_token.to_tokens(tokens);

                    match else_block.as_ref() {
                        ElseIf::Block(block) => {
                            block.to_backup(tokens, &unshadow);
                            break;
                        }
                        ElseIf::If(nested) => {
                            expr_if = nested;
                            continue;
                        }
                    }
                }
                None => break,
            }
        }

        unshadow.to_restore(tokens);
    }
}
