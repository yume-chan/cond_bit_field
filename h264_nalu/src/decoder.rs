use array_fill::array_fill;

use crate::nal_unit::{PictureParameterSet, SequenceParameterSet, UnsignedExpGolombCode};

pub struct Decoder {
    picture_parameter_sets: [Option<PictureParameterSet>; 256],
    sequence_parameter_sets: [Option<SequenceParameterSet>; 32],
}

impl Decoder {
    pub fn new() -> Self {
        Self {
            picture_parameter_sets: array_fill![None; 256],
            sequence_parameter_sets: array_fill![None; 32],
        }
    }

    pub fn set_picture_parameter_set(&mut self, picture_parameter_set: PictureParameterSet) {
        let id = picture_parameter_set.pic_parameter_set_id.0 as usize;
        self.picture_parameter_sets[id] = Some(picture_parameter_set);
    }

    pub fn set_sequence_parameter_set(&mut self, sequence_parameter_set: SequenceParameterSet) {
        let id = sequence_parameter_set.seq_parameter_set_id.0 as usize;
        self.sequence_parameter_sets[id] = Some(sequence_parameter_set);
    }

    pub fn find_picture_parameter_set(
        &self,
        id: UnsignedExpGolombCode,
    ) -> Option<&PictureParameterSet> {
        self.picture_parameter_sets[id.0 as usize].as_ref()
    }

    pub fn find_sequence_parameter_set(
        &self,
        id: UnsignedExpGolombCode,
    ) -> Option<&SequenceParameterSet> {
        self.sequence_parameter_sets[id.0 as usize].as_ref()
    }
}
