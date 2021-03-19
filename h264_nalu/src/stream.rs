use cond_bit_stream::{BitRead, BitReader};
use thiserror::Error;
use wasm_bindgen::prelude::*;

use crate::NalUnit;

#[derive(Error, Debug)]
pub enum NalUnitStreamError {
  #[error("Invalid emulation bytes")]
  InvalidEmulation,
  #[error("Stream contains no start code")]
  StartCodeNotFound,
  #[error("Stream has more than 3 continuous zero bytes")]
  TooManyZeros,
}

#[wasm_bindgen]
pub struct NalUnitStream {
  byte_stream: Box<[u8]>,
  start: usize,
}

#[wasm_bindgen]
impl NalUnitStream {
  #[wasm_bindgen(constructor)]
  pub fn new(byte_stream: Box<[u8]>) -> Self {
    NalUnitStream {
      byte_stream,
      start: 0,
    }
  }

  fn create_error<T>(err: impl std::fmt::Display) -> Result<T, JsValue> {
    Err(js_sys::Error::new(&*err.to_string()).into())
  }

  fn extract_nalu(&mut self, end: usize) -> Result<JsValue, JsValue> {
    let slice = &self.byte_stream[self.start..end];
    self.start = end;
    match BitReader::new(slice).read::<NalUnit>() {
      Ok(value) => serde_wasm_bindgen::to_value(&value).map_err(|err| err.into()),
      Err(err) => Self::create_error(err),
    }
  }

  pub fn next(&mut self) -> Result<JsValue, JsValue> {
    if self.byte_stream.len() == self.start {
      return Ok(JsValue::UNDEFINED);
    }

    let mut write_index = self.start;

    let mut zero_count = 0;
    let mut in_emulation = false;

    for i in self.start..self.byte_stream.len() {
      let byte = self.byte_stream[i];

      self.byte_stream[write_index] = byte;
      write_index += 1;

      if in_emulation {
        if byte > 0x03 {
          return Self::create_error(NalUnitStreamError::InvalidEmulation);
        }

        in_emulation = false;
        continue;
      }

      if byte == 0x00 {
        zero_count += 1;
        continue;
      }

      let zero_count = std::mem::replace(&mut zero_count, 0);

      if self.start == 0 {
        if zero_count >= 3 && byte == 0x01 {
          self.start = write_index;
          continue;
        }

        return Self::create_error(NalUnitStreamError::StartCodeNotFound);
      }

      if zero_count < 2 {
        continue;
      }

      if byte == 0x01 {
        return self.extract_nalu(write_index - zero_count - 1);
      }

      if zero_count > 2 {
        return Self::create_error(NalUnitStreamError::TooManyZeros);
      }

      match byte {
        0x02 => return Self::create_error(NalUnitStreamError::InvalidEmulation),
        0x03 => {
          write_index -= 1;
          in_emulation = true;
        }
        _ => {}
      }
    }

    if self.start == 0 {
      return Self::create_error(NalUnitStreamError::StartCodeNotFound);
    }

    if in_emulation {
      return Self::create_error(NalUnitStreamError::InvalidEmulation);
    }

    self.extract_nalu(self.byte_stream.len())
  }
}
