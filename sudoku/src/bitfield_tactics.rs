use super::{group_indices, Number, PlayingField};

pub struct BitfiedTacticsSolver {
    fields: [BitVector; 9 * 9],
}

#[derive(Clone, Copy)]
pub struct BitVector {
    mask: u16,
}

impl BitfiedTacticsSolver {
    pub fn new(play: &PlayingField) -> Self {
        let mut state = BitfiedTacticsSolver {
            fields: play.fields.map(From::from),
        };

        let groups = group_indices();

        for (idx, field) in play.fields.iter().enumerate() {
            let Number(Some(number)) = field else {
                continue;
            };

            let bit_mask = 1 << (number - 1);
            for group in &groups {
                if !group.contains(&idx) {
                    continue;
                }

                for &member_idx in group {
                    if member_idx == idx {
                        continue;
                    }

                    state.fields[member_idx].mask &= !bit_mask;
                }
            }
        }

        state
    }

    pub fn extract(&self) -> PlayingField {
        PlayingField {
            fields: self.fields.map(BitVector::to_exact_number),
        }
    }
}

impl BitVector {
    pub fn to_exact_number(self) -> Number {
        if self.mask.count_ones() != 1 {
            Number(None)
        } else {
            let bit_of_num = (self.mask.trailing_zeros() + 1) as u8;
            Number(Some(bit_of_num))
        }
    }
}

impl From<Number> for BitVector {
    fn from(value: Number) -> Self {
        BitVector {
            mask: match value {
                Number(Some(i)) => 1 << (i - 1),
                Number(None) => 0x1ff,
            },
        }
    }
}
