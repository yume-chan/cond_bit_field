#![allow(incomplete_features)]
#![feature(generic_associated_types)]

use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use syn::{parse_macro_input, token, Attribute, Block, Expr, ItemFn, Local, Pat, Signature, Stmt,
          Token, Type};

mod ty;
use ty::SimpleFieldType;

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
    pub ty: SimpleFieldType,
}

impl<'a> Field<'a> {
    pub fn to_field_tokens(&self, tokens: &mut TokenStream) {
        // #attrs0 #attrs1 pub #ident #colon_token #ty,
        tokens.append_all(self.attrs.0);
        tokens.append_all(self.attrs.1);
        token::Pub::default().to_tokens(tokens);
        self.ident.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
        token::Comma::default().to_tokens(tokens);
    }

    pub fn to_local_tokens(&self, tokens: &mut TokenStream) {
        // #let_token #ident #colon_token #ty #eq_token #init #semi_token
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
    type Iter<'a>: Iterator<Item = syn::Result<Field<'a>>>;

    fn iter_fields<'a>(&'a self) -> Self::Iter<'a>;
}

impl IterFields for Local {
    type Iter<'a> = std::option::IntoIter<syn::Result<Field<'a>>>;

    fn iter_fields<'a>(&'a self) -> Self::Iter<'a> {
        find_split(&self.attrs[..], |attr| attr.path.is_ident("field"))
            .map(|(attr, left, right)| {
                let (ident, colon_token, ty) = match &self.pat {
                    Pat::Type(pat_type) => match pat_type.pat.as_ref() {
                        Pat::Ident(ident) => {
                            Ok((&ident.ident, &pat_type.colon_token, &pat_type.ty))
                        }
                        pat @ _ => Err(syn::Error::new_spanned(pat, "field must have an ident.")),
                    },
                    pat @ _ => Err(syn::Error::new_spanned(pat, "field must have a type")),
                }?;
                Ok(Field {
                    local: self,
                    attrs: (left, right),
                    ident,
                    colon_token,
                    ty: SimpleFieldType::parse(ty.as_ref())?,
                    field_attr: attr,
                })
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
    type Item = syn::Result<Field<'a>>;

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

// let a: i5;
// to_field_tokens     -> pub a: i8,
// to_local_tokens     -> let a: i8;
// to_bit_field_tokens -> a = reader.read(5)?;

// if cond { let a: i5; }
// to_field_tokens     -> pub a: Option<i8>,
// to_local_tokens     -> let a: Option<i8> = None;
// to_bit_field_tokens -> a = Some(reader.read(5)?);

// if cond { let a: i5 = 5; }
// to_field_tokens     -> pub a: i8,
// to_local_tokens     -> let a: i8 = 5;
// to_bit_field_tokens -> a = reader.read(5)?;

// let a: SomeStruct;
// to_field_tokens     -> pub a: SomeStruct,
// to_local_tokens     -> let a: SomeStruct;
// to_bit_field_tokens -> a = reader.read(())?;

// let a: SomeStruct(value);
// to_field_tokens     -> pub a: SomeStruct,
// to_local_tokens     -> let a: SomeStruct;
// to_bit_field_tokens -> a = reader.read(value)?;

// #[skip] let _ = 3;
// to_bit_field_tokens -> reader.skip(3)?;

fn transform_bit_field(item: ItemFn) -> syn::Result<TokenStream> {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = item;

    let Signature {
        fn_token,
        ident,
        paren_token,
        inputs,
        ..
    } = sig;

    if let syn::ReturnType::Type(_, _) = &sig.output {
        Err(syn::Error::new_spanned(
            &sig.output,
            "bit_field fn must return nothing",
        ))?;
    }

    let fields = block.iter_fields().collect::<syn::Result<Vec<Field>>>()?;

    let mut stream = TokenStream::new();
    let tokens = &mut stream;

    // #attrs #vis struct #ident {
    //     #block.iter_fields().map(|field| field.to_field_tokens())
    // }
    tokens.append_all(attrs);
    vis.to_tokens(tokens);
    token::Struct::default().to_tokens(tokens);
    ident.to_tokens(tokens);
    token::Brace::default().surround(tokens, |tokens| {
        for field in &fields {
            field.to_field_tokens(tokens);
        }
    });

    // impl #ident {
    //     pub fn new(#inputs) -> Self {
    //         #block.iter_fields().map(|field| field.to_local_tokens())
    //         #block.stmts.map(|stmt| stmt.to_bit_field_tokens())
    //         Self {
    //             #block.iter_fields().map(|field| &field.ident) ,
    //         }
    //     }
    // }
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
            for field in &fields {
                field.to_local_tokens(tokens);
            }

            for stmt in &block.stmts {
                stmt.to_bit_field_tokens(tokens);
            }

            token::SelfType::default().to_tokens(tokens);
            token::Brace::default().surround(tokens, |tokens| {
                for field in &fields {
                    field.ident.to_tokens(tokens);
                    token::Comma::default().to_tokens(tokens);
                }
            });
        });
    });

    Ok(stream)
}

#[proc_macro_attribute]
pub fn bit_field(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as ItemFn);
    match transform_bit_field(item) {
        Ok(stream) => stream.into(),
        Err(error) => error.to_compile_error().into(),
    }
}
