use super::{group_indices, Number, PlayingField};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitfiedTacticsSolver {
    fields: [BitVector; 9 * 9],
}

static TIMESTAMP: std::sync::Mutex<Option<std::time::Instant>> = std::sync::Mutex::new(None);

impl BitfiedTacticsSolver {
    pub fn new(play: &PlayingField) -> Self {
        let mut this = BitfiedTacticsSolver {
            fields: play.fields.map(From::from),
        };

        for (idx, number) in play.iter_populated_fields() {
            this.set_field(idx, number);
        }

        for field in this.fields.iter() {
            // TODO: Unsolveable cases panic for now
            assert!(field.num_set_bits() != 0);
        }

        this
    }

    pub fn eliminate(&mut self, idx: usize, bit_mask: u16) -> bool {
        for group in group_indices() {
            if !group.contains(&idx) {
                continue;
            }

            for member_idx in group {
                if member_idx != idx {
                    // Only change the field when the bitmask would modify it
                    if (self.fields[member_idx].mask & bit_mask) != 0 {
                        self.fields[member_idx].mask &= !bit_mask;
                        match self.fields[member_idx].num_set_bits() {
                            // If we have only one possible value, we also have to eliminate its rows, column and cells.
                            1 => {
                                if !self.eliminate(member_idx, self.fields[member_idx].mask) {
                                    return false;
                                }
                            }
                            // If there is no possible value for this field, the whole state is incorrect.
                            0 => return false,
                            _ => (),
                        }
                    }
                }
            }
        }
        true
    }

    /// Sets field and updates all possibilities of all fields in related groups
    pub fn set_field(&mut self, idx: usize, number: u8) -> bool {
        let bit_mask = 1 << (number - 1);
        debug_assert!(
            self.fields[idx].mask & bit_mask != 0,
            "Illegal number {} for field with mask {:b}",
            number,
            self.fields[idx].mask,
        );

        self.fields[idx].mask = bit_mask;
        self.eliminate(idx, bit_mask)
    }

    pub fn try_solve(&self) -> Option<Self> {
        {
            let mut timestamp = TIMESTAMP.lock().unwrap();
            let timestamp = timestamp.get_or_insert_with(|| std::time::Instant::now());
            let now = std::time::Instant::now();
            if now - *timestamp > std::time::Duration::from_secs(10) {
                println!("{}", self.extract());
                *timestamp = now;
            }
        }

        if self.fields.iter().any(|f| f.num_set_bits() == 0) {
            return None;
        }

        if !self.extract().check_constraints() {
            println!("Unsolvable: {}", self.extract());
            self.extract().print_bad_constraints();
            //return None;
            panic!("Unsolvable");
        }

        let mut sorted_fields: Vec<(usize, BitVector)> = self
            .fields
            .iter()
            .copied()
            .enumerate()
            .filter(|(_i, f)| f.num_set_bits() != 1)
            .collect();
        sorted_fields.sort_by_key(|(_i, f)| f.num_set_bits());

        let (idx, bv) = match sorted_fields.first().copied() {
            // empty -> all solved
            None => return Some(self.clone()),
            // not solveable
            Some((_, field)) if field.num_set_bits() == 0 => {
                unreachable!();
            }
            // all good
            Some(f) => f,
        };

        debug_assert!(bv.num_set_bits() != 0);
        debug_assert!(bv.num_set_bits() != 1);

        for number in bv.iter_possible_numbers() {
            let mut new_field = self.clone();
            if !new_field.set_field(idx, number) {
                continue;
            };
            //dbg!(idx, number);
            if let Some(new_field) = new_field.try_solve() {
                return Some(new_field);
            }
        }

        // No number works
        None
    }

    pub fn extract(&self) -> PlayingField {
        PlayingField {
            fields: self.fields.map(BitVector::to_exact_number),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BitVector {
    mask: u16,
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

    pub fn num_set_bits(&self) -> u8 {
        u8::try_from(self.mask.count_ones())
            .expect("how do you fit more than 255 ones into an u16?!")
    }

    pub fn iter_possible_numbers(&self) -> PossibleNumberIterator {
        PossibleNumberIterator::new(*self)
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

pub struct PossibleNumberIterator {
    index: usize,
    bit_vector: BitVector,
}

impl PossibleNumberIterator {
    pub fn new(bit_vector: BitVector) -> Self {
        Self {
            index: 0,
            bit_vector,
        }
    }
}

impl Iterator for PossibleNumberIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        while self.bit_vector.mask & (1 << self.index) == 0 {
            self.index += 1;
            if self.index == 10 {
                return None;
            }
        }
        self.index += 1;
        Some(u8::try_from(self.index).expect("big mask no work my dude"))
    }
}
