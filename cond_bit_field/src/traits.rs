use std::iter::Iterator;

use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::data::Field;

pub type FieldIter<'a> = Box<dyn Iterator<Item = Field> + 'a>;

pub trait FlatFields {
  fn flat_fields(&self) -> FieldIter;
}

pub trait TypeToTokens {
  fn wrap_value(&self, tokens: TokenStream, ident: impl ToTokens);
}
