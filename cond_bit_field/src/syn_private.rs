/// Some methods copied from syn source code because they are not exported

pub mod pat {
    pub mod parsing {
        use syn::{parse::ParseStream, punctuated::Punctuated, Pat, PatOr, Result, Token};

        // https://docs.rs/syn/1.0.60/src/syn/pat.rs.html#736
        pub fn multi_pat_with_leading_vert(input: ParseStream) -> Result<Pat> {
            let leading_vert: Option<Token![|]> = input.parse()?;
            multi_pat_impl(input, leading_vert)
        }

        // https://docs.rs/syn/1.0.60/src/syn/pat.rs.html#741
        fn multi_pat_impl(input: ParseStream, leading_vert: Option<Token![|]>) -> Result<Pat> {
            let mut pat: Pat = input.parse()?;
            if leading_vert.is_some()
                || input.peek(Token![|]) && !input.peek(Token![||]) && !input.peek(Token![|=])
            {
                let mut cases = Punctuated::new();
                cases.push_value(pat);
                while input.peek(Token![|]) && !input.peek(Token![||]) && !input.peek(Token![|=]) {
                    let punct = input.parse()?;
                    cases.push_punct(punct);
                    let pat: Pat = input.parse()?;
                    cases.push_value(pat);
                }
                pat = Pat::Or(PatOr {
                    attrs: Vec::new(),
                    leading_vert,
                    cases,
                });
            }
            Ok(pat)
        }
    }
}

pub mod private {
    use syn::Attribute;

    // https://docs.rs/syn/1.0.60/src/syn/attr.rs.html#533
    pub fn attrs(outer: Vec<Attribute>, inner: Vec<Attribute>) -> Vec<Attribute> {
        let mut attrs = outer;
        attrs.extend(inner);
        attrs
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
