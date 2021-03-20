use std::collections::HashMap;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use syn::{braced,
          parse::{Parse, ParseStream},
          token, Result};

use crate::{data::Field,
            expr::Expr,
            traits::{FieldIter, FlatFields}};

pub struct ExprBlock {
    pub brace_token: token::Brace,
    pub stmts: Vec<Expr>,
}

impl ExprBlock {
    pub fn parse_within(input: ParseStream) -> Result<Vec<Expr>> {
        let mut stmts = Vec::new();
        while !input.is_empty() {
            stmts.push(input.parse()?);
        }
        Ok(stmts)
    }

    pub fn to_unshadow(&self, tokens: &mut TokenStream) {
        let unshadow = Unshadow::new(self);
        unshadow.to_dec(tokens);
        self.to_backup(tokens, &unshadow);
        unshadow.to_restore(tokens);
    }

    pub fn to_backup(&self, tokens: &mut TokenStream, unshadow: &Unshadow) {
        self.brace_token.surround(tokens, |tokens| {
            tokens.append_all(&self.stmts);
            unshadow.to_backup(self, tokens);
        });
    }
}

impl Parse for ExprBlock {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            brace_token: braced!(content in input),
            stmts: content.call(Self::parse_within)?,
        })
    }
}

impl FlatFields for ExprBlock {
    fn flat_fields(&self) -> FieldIter {
        Box::new(self.stmts.iter().flat_map(|x| x.flat_fields()))
    }
}

impl ToTokens for ExprBlock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.brace_token.surround(tokens, |tokens| {
            tokens.append_all(&self.stmts);
        });
    }
}

pub struct Unshadow {
    map: HashMap<Ident, (Field, Ident)>,
}

impl Unshadow {
    pub fn new(fields: &impl FlatFields) -> Self {
        let mut map: HashMap<Ident, (Field, Ident)> = HashMap::new();
        for field in fields.flat_fields() {
            let ident = field.ident.clone();
            let mangled = format_ident!(
                "{}_{}",
                &ident,
                thread_rng()
                    .sample_iter(Alphanumeric)
                    .take(4)
                    .map(char::from)
                    .collect::<String>()
            );
            map.insert(ident, (field, mangled));
        }

        Self { map }
    }

    pub fn to_dec(&self, tokens: &mut TokenStream) {
        for (_, (field, ident)) in self.map.iter() {
            let ty = &field.ty;
            tokens.extend(quote! {
              #[allow(non_snake_case)]
              let mut #ident: #ty =
            });
            ty.to_default(tokens);
            tokens.extend(quote! {;});
        }
    }

    pub fn to_backup(&self, fields: &impl FlatFields, tokens: &mut TokenStream) {
        for field in fields.flat_fields() {
            let src = &field.ident;
            let (field, dest) = self.map.get(src).unwrap();
            field.ty.to_backup(tokens, src, dest);
        }
    }

    pub fn to_restore(&self, tokens: &mut TokenStream) {
        for (dest, (_, src)) in self.map.iter() {
            tokens.extend(quote! {let #dest = #src;})
        }
    }
}
