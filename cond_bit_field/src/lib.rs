#![allow(incomplete_features)]
#![feature(generic_associated_types)]

use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use syn::{parse_macro_input, token, Attribute, Block, Expr, ItemFn, Local, Pat, Signature, Stmt,
          Token, Type};

fn find_split<T, F: FnMut(&T) -> bool>(slice: &[T], predicate: F) -> Option<(&T, &[T], &[T])> {
    slice
        .iter()
        .position(predicate)
        .map(|index| (&slice[index], &slice[..index], &slice[index + 1..]))
}

struct Field<'a> {
    pub local: &'a Local,
    pub attrs: (&'a [Attribute], &'a [Attribute]),
    pub field_attr: &'a Attribute,
    pub ident: &'a Ident,
    pub colon_token: &'a Token![:],
    pub ty: &'a Box<Type>,
}

impl<'a> Field<'a> {
    pub fn to_field_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.attrs.0);
        tokens.append_all(self.attrs.1);
        token::Pub::default().to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
        token::Comma::default().to_tokens(tokens);
    }

    pub fn to_local_tokens(&self, tokens: &mut TokenStream) {
        tokens.append_all(self.attrs.0);
        tokens.append_all(self.attrs.1);
        self.local.let_token.to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
        if let Some((eq_token, init)) = &self.local.init {
            eq_token.to_tokens(tokens);
            init.to_tokens(tokens);
        }
        self.local.semi_token.to_tokens(tokens);
    }
}

trait IterFields {
    type Iter<'a>: Iterator<Item = Field<'a>>;

    fn iter_fields<'a>(&'a self) -> Self::Iter<'a>;
}

impl IterFields for Local {
    type Iter<'a> = std::option::IntoIter<Field<'a>>;

    fn iter_fields<'a>(&'a self) -> Self::Iter<'a> {
        let pat_is_ident = match &self.pat {
            Pat::Type(ty) => match ty.pat.as_ref() {
                Pat::Ident(ident) => Some((&ident.ident, &ty.colon_token, &ty.ty)),
                _ => None,
            },
            _ => None,
        };
        pat_is_ident
            .and_then(|(ident, colon_token, ty)| {
                find_split(&self.attrs[..], |attr| attr.path.is_ident("field")).map(
                    |(attr, left, right)| Field {
                        local: self,
                        attrs: (left, right),
                        ident,
                        colon_token,
                        ty,
                        field_attr: attr,
                    },
                )
            })
            .into_iter()
    }
}

enum StmtFields<'a> {
    Local(<Local as IterFields>::Iter<'a>),
    None,
}

impl<'a> StmtFields<'a> {
    pub fn new(stmt: &'a Stmt) -> Self {
        match stmt {
            Stmt::Local(local) => Self::Local(local.iter_fields()),
            Stmt::Expr(expr) | Stmt::Semi(expr, _) => Self::None,
            Stmt::Item(_) => Self::None,
        }
    }
}

impl<'a> Iterator for StmtFields<'a> {
    type Item = Field<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Local(local) => local.next(),
            Self::None => None,
        }
    }
}

impl IterFields for Stmt {
    type Iter<'a> = StmtFields<'a>;

    fn iter_fields<'a>(&'a self) -> Self::Iter<'a> {
        StmtFields::new(self)
    }
}

impl IterFields for Block {
    type Iter<'a> = std::iter::FlatMap<
        std::slice::Iter<'a, Stmt>,
        StmtFields<'a>,
        fn(&'a Stmt) -> StmtFields<'a>,
    >;

    fn iter_fields<'a>(&'a self) -> Self::Iter<'a> {
        self.stmts.iter().flat_map(|stmt| stmt.iter_fields())
    }
}

trait ToBitFieldTokens {
    fn to_bit_field_tokens(&self, tokens: &mut TokenStream);
}

impl ToBitFieldTokens for Local {
    fn to_bit_field_tokens(&self, tokens: &mut TokenStream) {}
}

impl ToBitFieldTokens for Stmt {
    fn to_bit_field_tokens(&self, tokens: &mut TokenStream) {
        match self {
            // Stmt::Local(local) => local.to_bit_field_tokens(tokens),
            _ => {}
        }
    }
}

impl ToBitFieldTokens for Block {
    fn to_bit_field_tokens(&self, tokens: &mut TokenStream) {
        self.brace_token.surround(tokens, |tokens| {
            for stmt in &self.stmts {
                stmt.to_bit_field_tokens(tokens);
            }
        })
    }
}

#[proc_macro_attribute]
pub fn bit_field(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(input as ItemFn);
    let Signature {
        fn_token,
        ident,
        paren_token,
        inputs,
        ..
    } = sig;

    if let syn::ReturnType::Type(_, _) = &sig.output {
        return proc_macro::TokenStream::from(
            syn::Error::new_spanned(&sig.output, "bit_field fn must return nothing")
                .into_compile_error(),
        );
    }

    let mut stream = TokenStream::new();
    let tokens = &mut stream;
    tokens.append_all(attrs);
    vis.to_tokens(tokens);
    token::Struct::default().to_tokens(tokens);
    ident.to_tokens(tokens);
    token::Brace::default().surround(tokens, |tokens| {
        for field in block.iter_fields() {
            field.to_field_tokens(tokens);
        }
    });

    // translate_block(block.as_ref(), tokens);

    token::Impl::default().to_tokens(tokens);
    ident.to_tokens(tokens);
    token::Brace::default().surround(tokens, |tokens| {
        token::Pub::default().to_tokens(tokens);
        fn_token.to_tokens(tokens);
        tokens.append(Ident::new("new", Span::call_site()));
        paren_token.surround(tokens, |tokens| inputs.to_tokens(tokens));
        token::RArrow::default().to_tokens(tokens);
        token::SelfType::default().to_tokens(tokens);
        token::Brace::default().surround(tokens, |tokens| {
            for field in block.iter_fields() {
                field.to_local_tokens(tokens);
            }

            token::SelfType::default().to_tokens(tokens);
            token::Brace::default().surround(tokens, |tokens| {
                for field in block.iter_fields() {
                    field.ident.to_tokens(tokens);
                    token::Comma::default().to_tokens(tokens);
                }
            });
        });
    });

    stream.into()
}

struct Foo {
    value: Vec<u8>,
}

impl IntoIterator for Foo {
    type Item = u8;
    type IntoIter = std::iter::Map<std::vec::IntoIter<u8>, fn(u8) -> u8>;
    fn into_iter(self) -> Self::IntoIter {
        self.value.into_iter().map(|x| x + 1)
    }
}
