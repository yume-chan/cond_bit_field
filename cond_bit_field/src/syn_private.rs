pub mod parsing {
  use syn::{parse::ParseStream, punctuated::Punctuated, Pat, PatOr, Result, Token};

  pub fn multi_pat_with_leading_vert(input: ParseStream) -> Result<Pat> {
    let leading_vert: Option<Token![|]> = input.parse()?;
    multi_pat_impl(input, leading_vert)
  }

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

pub mod private {
  use syn::Attribute;

  pub fn attrs(outer: Vec<Attribute>, inner: Vec<Attribute>) -> Vec<Attribute> {
    let mut attrs = outer;
    attrs.extend(inner);
    attrs
  }
}
