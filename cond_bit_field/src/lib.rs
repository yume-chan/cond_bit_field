use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{ToTokens, TokenStreamExt};
use syn::{parse_macro_input, token, Block, Expr, ItemFn, Local, Pat, Signature, Stmt};

fn find_split<T, F: FnMut(&T) -> bool>(slice: &[T], predicate: F) -> (&[T], Option<&T>, &[T]) {
    match slice.iter().position(predicate) {
        Some(index) => (&slice[..index], Some(&slice[index]), &slice[index + 1..]),
        None => (slice, None, &[]),
    }
}

fn translate_expr(expr: &Expr, tokens: &mut proc_macro2::TokenStream) {}

fn translate_stmt(stmt: &Stmt, tokens: &mut proc_macro2::TokenStream) {
    match stmt {
        Stmt::Local(Local { attrs, pat, .. }) => {
            if let (left, Some(_), right) =
                find_split(&attrs[..], |attr| attr.path.is_ident("field"))
            {
                match pat {
                    Pat::Type(pat) => {
                        tokens.append_all(left);
                        tokens.append_all(right);
                        pat.to_tokens(tokens);
                        token::Comma::default().to_tokens(tokens);
                    }
                    _ => unreachable!(),
                }
            }
        }
        Stmt::Expr(expr) | Stmt::Semi(expr, _) => translate_expr(expr, tokens),
        Stmt::Item(_) => {}
    }
}

fn translate_block(block: &Block, tokens: &mut proc_macro2::TokenStream) {
    block.brace_token.surround(tokens, |tokens| {
        for stmt in &block.stmts {
            translate_stmt(stmt, tokens);
        }
    });
}

#[proc_macro_attribute]
pub fn bit_field(_args: TokenStream, input: TokenStream) -> TokenStream {
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

    let mut tokens = proc_macro2::TokenStream::new();
    tokens.append_all(attrs);
    vis.to_tokens(&mut tokens);
    token::Struct::default().to_tokens(&mut tokens);
    ident.to_tokens(&mut tokens);
    translate_block(block.as_ref(), &mut tokens);

    token::Impl::default().to_tokens(&mut tokens);
    ident.to_tokens(&mut tokens);
    token::Brace::default().surround(&mut tokens, |tokens| {
        token::Pub::default().to_tokens(tokens);
        fn_token.to_tokens(tokens);
        tokens.append(Ident::new("new", Span::call_site()));
        paren_token.surround(tokens, |tokens| inputs.to_tokens(tokens));
        token::RArrow::default().to_tokens(tokens);
        token::SelfType::default().to_tokens(tokens);
    });

    tokens.into()
}
