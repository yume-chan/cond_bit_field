use thiserror::Error;
use wasm_bindgen::prelude::*;

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
  pub fn new(byte_stream: Box<[u8]>) -> Self {
    NalUnitStream {
      byte_stream,
      start: 0,
    }
  }

  fn create_error(err: NalUnitStreamError) -> JsValue {
    JsValue::from(format!("{:?}", err))
  }

  fn extract_nalu(&mut self, end: usize) -> Box<[u8]> {
    let slice = self.byte_stream[self.start..end]
      .to_vec()
      .into_boxed_slice();
    self.start = end;
    slice
  }

  pub fn next(&mut self) -> Result<Option<Box<[u8]>>, JsValue> {
    if self.byte_stream.len() == self.start {
      return Ok(None);
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
          return Err(Self::create_error(NalUnitStreamError::InvalidEmulation));
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

        return Err(Self::create_error(NalUnitStreamError::StartCodeNotFound));
      }

      if zero_count < 2 {
        continue;
      }

      if byte == 0x01 {
        return Ok(Some(self.extract_nalu(write_index - zero_count - 1)));
      }

      if zero_count > 2 {
        return Err(Self::create_error(NalUnitStreamError::TooManyZeros));
      }

      match byte {
        0x02 => return Err(Self::create_error(NalUnitStreamError::InvalidEmulation)),
        0x03 => {
          write_index -= 1;
          in_emulation = true;
        }
        _ => {}
      }
    }

    if self.start == 0 {
      return Err(Self::create_error(NalUnitStreamError::StartCodeNotFound));
    }

    if in_emulation {
      return Err(Self::create_error(NalUnitStreamError::InvalidEmulation));
    }

    Ok(Some(self.extract_nalu(self.byte_stream.len())))
  }
}
