use std::iter;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{bracketed, parenthesized,
          parse::{Parse, ParseStream, Parser},
          punctuated::Punctuated,
          token, Attribute, LitInt, Result, Token, Visibility};

use crate::{block::ExprBlock,
            traits::{FieldIter, FlatFields},
            ty::{ComplexType, Type}};

pub struct Skip {
    pub underscore_token: token::Underscore,

    pub colon_token: token::Colon,

    pub size: LitInt,

    pub semicolon_token: token::Semi,
}

impl Parse for Skip {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Skip {
            underscore_token: input.parse()?,
            colon_token: input.parse()?,
            size: input.parse()?,
            semicolon_token: input.parse()?,
        })
    }
}

impl FlatFields for Skip {
    fn flat_fields(&self) -> FieldIter {
        Box::new(iter::empty())
    }
}

impl ToTokens for Skip {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let size = &self.size;
        tokens.extend(quote! {stream.skip(#size)?;});
    }
}

pub struct ParseParen<T> {
    pub paren_token: token::Paren,
    pub content: T,
}

impl<T: Parse> Parse for ParseParen<T> {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let paren_token = parenthesized!(content in input);
        Ok(Self {
            paren_token,
            content: content.parse()?,
        })
    }
}

impl<T> ParseParen<T> {
    fn parse_with<F: FnOnce(ParseStream) -> Result<T>>(input: TokenStream, f: F) -> Result<Self> {
        let func = |input: ParseStream| -> Result<Self> {
            let content;
            let paren_token = parenthesized!(content in input);
            Ok(Self {
                paren_token,
                content: f(&content)?,
            })
        };
        Parser::parse2(func, input)
    }
}

#[derive(Clone)]
pub struct Field {
    /// Attributes tagged on the field.
    pub attrs: Vec<Attribute>,

    /// Visibility of the field.
    pub vis: Visibility,

    /// Name of the field, if any.
    ///
    /// Fields of tuple structs have no names.
    pub ident: Ident,

    pub colon_token: token::Colon,

    /// Type of the field.
    pub ty: ComplexType,

    pub params: Option<Punctuated<syn::Expr, token::Comma>>,

    pub default: Option<syn::Expr>,

    pub semicolon_token: token::Semi,
}

impl Field {
    pub fn into_option(mut self) -> Self {
        if self.default == None {
            self.ty = ComplexType::Option(Box::new(self.ty));
        }
        self
    }

    pub fn to_initializer(&self) -> TokenStream {
        match self.ty.inner_most() {
            Type::Bool { .. } => quote! {stream.read_bit()?},
            Type::Number { size, .. } => {
                if let Some(params) = &self.params {
                    quote! {stream.read(#params)?}
                } else {
                    quote! {stream.read(#size)?}
                }
            }
            Type::Struct(_) => {
                if let Some(params) = &self.params {
                    quote! {stream.read((#params))?}
                } else {
                    quote! {stream.read(())?}
                }
            }
        }
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let vis = input.parse()?;
        let ident = input.parse()?;
        let colon_token = input.parse()?;
        let ty = ComplexType::Simple(input.parse()?);

        let mut params = None;
        if input.peek(token::Bracket) {
            let content;
            let _ = bracketed!(content in input);
            params = Some(
                Punctuated::<syn::Expr, token::Comma>::parse_terminated_with(
                    &content,
                    syn::Expr::parse_without_eager_brace,
                )?,
            );
        }

        let mut default = None;
        if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            default = Some(input.parse()?);
        }

        Ok(Field {
            attrs: Vec::new(),
            vis,
            ident,
            colon_token,
            ty,
            params,
            default,
            semicolon_token: input.parse()?,
        })
    }
}

impl FlatFields for Field {
    fn flat_fields(&self) -> FieldIter {
        Box::new(iter::once(self.clone()))
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Field {
            ident,
            ty,
            semicolon_token,
            ..
        } = self;

        let ty = ty.inner_most();
        let initializer = self.to_initializer();
        tokens.extend(quote! {let #ident: #ty = #initializer #semicolon_token});
    }
}

pub struct Struct {
    pub attrs: Vec<Attribute>,
    pub extra_args: Option<Punctuated<SimplePatType, token::Comma>>,
    pub vis: Visibility,
    pub struct_token: Token![struct],
    pub ident: Ident,
    pub fields: ExprBlock,
}

pub struct SimplePatType {
    pub ident: Ident,
    pub colon_token: Token![:],
    pub ty: syn::Type,
}

impl ToTokens for SimplePatType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.ident.to_tokens(tokens);
        self.colon_token.to_tokens(tokens);
        self.ty.to_tokens(tokens);
    }
}

impl Parse for Struct {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let mut extra_args_attribute: Option<Attribute> = None;
        let attrs = attrs
            .into_iter()
            .filter(|x| {
                let segments = &x.path.segments;
                if segments.len() == 1 {
                    let first = segments.first().unwrap();
                    if first.ident.to_string() == "extra_args" {
                        extra_args_attribute = Some(x.clone());
                        return false;
                    }
                }
                true
            })
            .collect::<Vec<_>>();

        let extra_args = match extra_args_attribute {
            Some(attribute) => {
                let args = ParseParen::parse_with(attribute.tokens, |input| {
                    Punctuated::<SimplePatType, token::Comma>::parse_terminated_with(
                        input,
                        |input| {
                            Ok(SimplePatType {
                                ident: input.parse()?,
                                colon_token: input.parse()?,
                                ty: input.parse()?,
                            })
                        },
                    )
                })?;
                Some(args.content)
            }
            None => None,
        };

        Ok(Struct {
            attrs,
            extra_args,
            vis: input.parse()?,
            struct_token: input.parse()?,
            ident: input.parse()?,
            fields: input.parse()?,
        })
    }
}

impl ToTokens for Struct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            attrs,
            extra_args,
            vis,
            struct_token,
            ident,
            fields,
        } = self;

        tokens.extend(quote! {
          #(#attrs)* #vis #struct_token #ident
        });

        fields.brace_token.surround(tokens, |tokens| {
            tokens.append_all(fields.flat_fields().map(
                |Field {
                     attrs,
                     vis,
                     ident,
                     colon_token,
                     ty,
                     ..
                 }| {
                    quote! {
                      #(#attrs)* #vis #ident #colon_token #ty,
                    }
                },
            ));
        });

        let initializers = &fields.stmts;
        let field_names = fields.flat_fields().map(|x| x.ident);

        let mut arg_types = Vec::<TokenStream>::new();
        let mut arg_names = Vec::<&Ident>::new();

        if let Some(args) = &extra_args {
            for SimplePatType { ident, ty, .. } in args {
                if let syn::Type::Reference(syn::TypeReference {
                    mutability, elem, ..
                }) = &ty
                {
                    arg_types.push(quote! {&'a #mutability #elem});
                } else {
                    arg_types.push(quote! {#ty});
                }

                arg_names.push(ident);
            }
        }

        let arg_destruction = quote! {let (#(#arg_names),*) = args;};

        tokens.extend(quote! {
            impl<'a> bit_stream::BitField<'a> for #ident {
                type Args = (#(#arg_types),*);

                fn read  (stream: &mut bit_stream::BitStream, args: Self::Args) -> bit_stream::Result<Self> {
                    #arg_destruction
                    #(#initializers)*
                    Ok(Self {
                        #(#field_names),*
                    })
                }
            }
        });
    }
}
