use std::iter::Iterator;

use proc_macro2::TokenStream;
use quote::ToTokens;

use crate::data::Field;

pub type FieldIter<'a> = Box<dyn Iterator<Item = &'a Field> + 'a>;
pub type FieldIterMut<'a> = Box<dyn Iterator<Item = &'a mut Field> + 'a>;

pub trait FlatFields {
  fn flat_fields(&self) -> FieldIter;
  fn flat_fields_mut(&mut self) -> FieldIterMut;
}

pub trait TypeToTokens {
  fn wrap_value(&self, tokens: TokenStream, ident: impl ToTokens);
}
