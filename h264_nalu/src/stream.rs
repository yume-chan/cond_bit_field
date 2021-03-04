use thiserror::Error;

#[derive(Error, Debug)]
pub enum NalUnitStreamError {
  #[error("Invalid emulation bytes")]
  InvalidEmulation,
  #[error("Stream contains no start code")]
  StartCodeNotFound,
  #[error("Stream has more than 3 continuous zero bytes")]
  TooManyZeros,
}

pub struct NalUnitStream<'a> {
  byte_stream: &'a mut [u8],
  started: bool,
}

impl<'a> NalUnitStream<'a> {
  pub fn new(byte_stream: &'a mut [u8]) -> Self {
    NalUnitStream {
      byte_stream,
      started: false,
    }
  }

  fn extract_nalu(&mut self, end: usize) -> &'a [u8] {
    let byte_stream = std::mem::replace(&mut self.byte_stream, &mut []);
    let (left, right) = byte_stream.split_at_mut(end);
    self.byte_stream = right;
    return left;
  }

  pub fn next(&mut self) -> Result<Option<&'a [u8]>, NalUnitStreamError> {
    if self.byte_stream.len() == 0 {
      return Ok(None);
    }

    let mut write_index = 0;

    let mut zero_count = 0;
    let mut in_emulation = false;

    for i in 0..self.byte_stream.len() {
      let byte = self.byte_stream[i];

      self.byte_stream[write_index] = byte;
      write_index += 1;

      if in_emulation {
        if byte > 0x03 {
          return Err(NalUnitStreamError::InvalidEmulation);
        }

        in_emulation = false;
        continue;
      }

      if byte == 0x00 {
        zero_count += 1;
        continue;
      }

      let zero_count = std::mem::replace(&mut zero_count, 0);

      if !self.started {
        if zero_count >= 3 && byte == 0x01 {
          self.started = true;
          write_index = 0;
          continue;
        }

        return Err(NalUnitStreamError::StartCodeNotFound);
      }

      if zero_count < 2 {
        continue;
      }

      if byte == 0x01 {
        return Ok(Some(self.extract_nalu(write_index - zero_count - 1)));
      }

      if zero_count > 2 {
        return Err(NalUnitStreamError::TooManyZeros);
      }

      match byte {
        0x02 => return Err(NalUnitStreamError::InvalidEmulation),
        0x03 => {
          write_index -= 1;
          in_emulation = true;
        }
        _ => {}
      }
    }

    if !self.started {
      return Err(NalUnitStreamError::StartCodeNotFound);
    }

    if in_emulation {
      return Err(NalUnitStreamError::InvalidEmulation);
    }

    Ok(Some(self.extract_nalu(self.byte_stream.len())))
  }
}
