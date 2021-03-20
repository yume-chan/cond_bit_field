use std::iter::Iterator;

use crate::data::Field;

pub type FieldIter<'a> = Box<dyn Iterator<Item = Field> + 'a>;

pub trait FlatFields {
    fn flat_fields(&self) -> FieldIter;
}
