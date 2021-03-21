use std::iter;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{parenthesized,
          parse::{Parse, ParseStream, Parser},
          punctuated::Punctuated,
          token, Attribute, LitInt, PatType, Result, Token, Visibility};

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

    pub size: Option<syn::Expr>,

    pub default: Option<syn::Expr>,

    pub extra_args: Option<Punctuated<syn::Expr, token::Comma>>,

    /// Visibility of the field.
    pub vis: Visibility,

    /// Name of the field, if any.
    ///
    /// Fields of tuple structs have no names.
    pub ident: Ident,

    pub colon_token: token::Colon,

    /// Type of the field.
    pub ty: ComplexType,

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
        match self.size {
            Some(ref size) => quote! {stream.read_sized(#size)?},
            None => match self.ty.inner_most() {
                Type::Bool { .. } => quote! {stream.read_bit()?},
                Type::Number { size, .. } => quote! {stream.read_sized(#size)?},
                Type::Struct(ty) => match &self.extra_args {
                    Some(extra_args) => quote! {#ty::read(stream, #extra_args)?},
                    None => quote! {stream.read()?},
                },
            },
        }
    }

    pub fn process_fake_attrs(&mut self) -> Result<()> {
        let attrs = std::mem::take(&mut self.attrs);

        let mut size_attribute: Option<Attribute> = None;
        let mut default_attribute: Option<Attribute> = None;
        let mut extra_args_attribute: Option<Attribute> = None;

        let attrs = attrs
            .into_iter()
            .filter(|x| {
                let segments = &x.path.segments;
                if segments.len() == 1 {
                    let first = segments.first().unwrap();
                    match first.ident.to_string().as_str() {
                        "size" => size_attribute = Some(x.clone()),
                        "default" => default_attribute = Some(x.clone()),
                        "extra_args" => extra_args_attribute = Some(x.clone()),
                        _ => return true,
                    }
                    return false;
                }
                true
            })
            .collect::<Vec<_>>();

        let size = match size_attribute {
            Some(attribute) => {
                let args =
                    ParseParen::parse_with(attribute.tokens, syn::Expr::parse_without_eager_brace)?;
                Some(args.content)
            }
            None => None,
        };

        let default = match default_attribute {
            Some(attribute) => {
                let args =
                    ParseParen::parse_with(attribute.tokens, syn::Expr::parse_without_eager_brace)?;
                Some(args.content)
            }
            None => None,
        };

        let extra_args = match extra_args_attribute {
            Some(attribute) => {
                let args = ParseParen::parse_with(attribute.tokens, |input| {
                    Punctuated::<syn::Expr, token::Comma>::parse_terminated_with(
                        input,
                        syn::Expr::parse_without_eager_brace,
                    )
                })?;
                Some(args.content)
            }
            None => None,
        };

        self.attrs = attrs;
        self.size = size;
        self.default = default;
        self.extra_args = extra_args;

        Ok(())
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Field {
            attrs: Vec::new(),
            size: None,
            default: None,
            extra_args: None,
            vis: input.parse()?,
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: ComplexType::Simple(input.parse()?),
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
    pub extra_args: Option<Punctuated<PatType, token::Comma>>,
    pub vis: Visibility,
    pub struct_token: Token![struct],
    pub ident: Ident,
    pub fields: ExprBlock,
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
                    Punctuated::<PatType, token::Comma>::parse_terminated_with(input, |input| {
                        Ok(PatType {
                            attrs: Vec::new(),
                            pat: input.parse()?,
                            colon_token: input.parse()?,
                            ty: input.parse()?,
                        })
                    })
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

        match extra_args {
            Some(extra_args) => {
                tokens.extend(quote! {
                    impl #ident {
                        pub fn read(stream: &mut bit_stream::BitStream, #extra_args) -> bit_stream::Result<Self> {
                            #(#initializers)*
                            Ok(Self {
                                #(#field_names),*
                            })
                        }
                    }
                });
            }
            None => {
                tokens.extend(quote! {
                    impl bit_stream::BitField for #ident {
                        fn read(stream: &mut bit_stream::BitStream) -> bit_stream::Result<Self> {
                            #(#initializers)*
                            Ok(Self {
                                #(#field_names),*
                            })
                        }
                    }
                });
            }
        };
    }
}
