use crate::SignedExpGolombCode;
use cond_bit_stream::{BitRead, Result, SizedBitField};
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct ScalingList {
  pub list: Vec<u8>,
  pub use_default_scaling_matrix_flag: bool,
}

impl SizedBitField for ScalingList {
  fn read_sized(reader: &mut impl BitRead, size: u8) -> Result<Self> {
    let mut list: Vec<u8> = Vec::with_capacity(size as usize);

    let mut last_scale = {
      let delta_scale: SignedExpGolombCode = reader.read()?;
      ((8 + delta_scale + 256) % 256) as u8
    };

    if last_scale == 0 {
      for _ in 0..size {
        list.push(8);
      }
      return Ok(Self {
        list,
        use_default_scaling_matrix_flag: true,
      });
    }

    let mut j = 1u8;
    loop {
      let delta_scale: SignedExpGolombCode = reader.read()?;
      let next_scale: u8 = ((last_scale as i64 + delta_scale.0 + 256) % 256) as u8;

      j += 1;

      if next_scale != 0 {
        last_scale = next_scale;
        list.push(last_scale);
      } else {
        list.push(last_scale);
        break;
      }

      if j == size {
        return Ok(Self {
          list,
          use_default_scaling_matrix_flag: false,
        });
      }
    }

    while j < size {
      list.push(last_scale);
    }

    Ok(Self {
      list,
      use_default_scaling_matrix_flag: false,
    })
  }
}
