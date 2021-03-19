extern crate proc_macro;

use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Error, Fields};

#[proc_macro_derive(NewNumber)]
pub fn new_number(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let derive = parse_macro_input!(input as DeriveInput);
  match derive.data {
    Data::Struct(ref r#struct) => match r#struct.fields {
      Fields::Unnamed(ref fields) => {
        if fields.unnamed.len() == 1 {
          let struct_name = &derive.ident;
          let field = &fields.unnamed[0];
          let ty = &field.ty;
          proc_macro::TokenStream::from(quote! {
              impl<T> core::ops::Add<T> for #struct_name
                  where #ty: core::ops::Add<T, Output = #ty>
              {
                  type Output = Self;

                  #[must_use]
                  #[inline]
                  fn add(self, rhs: T) -> Self::Output {
                      Self(self.0.add(rhs))
                  }
              }

              impl core::ops::Add<#struct_name> for #ty {
                  type Output = Self;

                  #[must_use]
                  #[inline]
                  fn add(self, rhs: #struct_name) -> Self::Output {
                      self.add(rhs.0)
                  }
              }

              impl<T> core::ops::Sub<T> for #struct_name
                  where #ty: core::ops::Sub<T, Output = #ty>
              {
                  type Output = Self;

                  #[must_use]
                  #[inline]
                  fn sub(self, rhs: T) -> Self::Output {
                      Self(self.0.sub(rhs))
                  }
              }

              impl core::ops::Sub<#struct_name> for #ty {
                  type Output = Self;

                  #[must_use]
                  #[inline]
                  fn sub(self, rhs: #struct_name) -> Self::Output {
                      self.sub(rhs.0)
                  }
              }

              impl<T> core::ops::Rem<T> for #struct_name
                  where #ty: core::ops::Rem<T, Output = #ty>
              {
                  type Output = Self;

                  #[must_use]
                  #[inline]
                  fn rem(self, rhs: T) -> Self::Output {
                      Self(self.0.rem(rhs))
                  }
              }

              impl core::ops::Rem<#struct_name> for #ty {
                  type Output = Self;

                  #[must_use]
                  #[inline]
                  fn rem(self, rhs: #struct_name) -> Self::Output {
                      self.rem(rhs.0)
                  }
              }

              impl<T> core::ops::Shr<T> for #struct_name
                  where #ty: core::ops::Shr<T, Output = #ty>
              {
                  type Output = Self;

                  #[must_use]
                  #[inline]
                  fn shr(self, rhs: T) -> Self::Output {
                      Self(self.0.shr(rhs))
                  }
              }

              impl core::ops::Shr<#struct_name> for #ty {
                  type Output = Self;

                  #[must_use]
                  #[inline]
                  fn shr(self, rhs: #struct_name) -> Self::Output {
                      self.shr(rhs.0)
                  }
              }

              impl core::cmp::PartialEq<#struct_name> for #ty {
                  #[inline]
                  fn eq(&self, other: &#struct_name) -> bool {
                      self.eq(&other.0)
                  }
              }

              impl core::cmp::PartialEq<#ty> for #struct_name {
                  #[inline]
                  fn eq(&self, other: &#ty) -> bool {
                      self.0.eq(other)
                  }
              }

              impl core::cmp::PartialOrd<#struct_name> for #ty {
                  #[inline]
                  fn partial_cmp(&self, other: &#struct_name) -> Option<core::cmp::Ordering> {
                      self.partial_cmp(&other.0)
                  }
              }

              impl core::cmp::PartialOrd<#ty> for #struct_name {
                  #[inline]
                  fn partial_cmp(&self, other: &#ty) -> Option<core::cmp::Ordering> {
                      self.0.partial_cmp(other)
                  }
              }
          })
        } else {
          Error::new_spanned(&fields.unnamed, "Expect only one field")
            .into_compile_error()
            .into()
        }
      }
      _ => Error::new_spanned(&r#struct.fields, "Expect unnamed fields")
        .into_compile_error()
        .into(),
    },
    _ => Error::new_spanned(derive, "Expect a struct")
      .into_compile_error()
      .into(),
  }
}
