use core::fmt;

fn main() {
    // let pf = PlayingField::default();
    let pf = PlayingField::new(
        "004300209005009001070060043006002087190007400050083000600000105003508690042910300",
    )
    .unwrap();

    // Solved
    //let pf = PlayingField::new(
    //    "123456789456789123789123456234567891567891234891234567345678912678912345912345678",
    //)
    //.unwrap();
    println!("{}", pf);

    println!("Checked Constraints: {}", pf.check_constraints());
    println!("Checked Completed: {}", pf.is_complete());
    println!("Checked Solved: {}", pf.is_solved());

    // for g in group_indices() {
    //     let mut pf = PlayingField::default();
    //     pf.set_group(g, Coloring(Some(8)));
    //     println!("{}", pf);
    // }
}

#[derive(Clone, PartialEq, Eq)]
struct PlayingField {
    fields: [Coloring; 9 * 9],
}

fn group_indices() -> Vec<[usize; 9]> {
    let mut res = vec![];
    const IOTA: [usize; 9] = [0, 1, 2, 3, 4, 5, 6, 7, 8];
    const CELL: [usize; 9] = [0, 1, 2, 9, 10, 11, 18, 19, 20];

    for row in 0..9 {
        res.push(IOTA.map(|n| n + row * 9));
    }

    for col in 0..9 {
        res.push(IOTA.map(|n| n * 9 + col));
    }

    for row in 0..3 {
        for col in 0..3 {
            let base_idx = row * 9 * 3 + col * 3;
            res.push(CELL.map(|n| n + base_idx));
        }
    }

    res
}

impl PlayingField {
    pub fn new(f: &str) -> Result<PlayingField, String> {
        let mut fields = vec![];
        for c in f.chars() {
            fields.push(Coloring::new(c)?);
        }
        Ok(PlayingField {
            fields: fields.try_into().unwrap(),
        })
    }

    pub fn set_group(&mut self, group: [usize; 9], color: Coloring) {
        for idx in group {
            self.fields[idx] = color;
        }
    }

    pub fn check_constraints(&self) -> bool {
        for g in group_indices() {
            let mut numbers: Vec<_> = g.into_iter().filter_map(|i| self.fields[i].0).collect();
            numbers.sort();
            if numbers.windows(2).any(|w| w[0] == w[1]) {
                return false;
            }
        }
        true
    }

    pub fn is_complete(&self) -> bool {
        self.fields.iter().all(|c| c.0.is_some())
    }

    pub fn is_solved(&self) -> bool {
        self.is_complete() && self.check_constraints()
    }
}

/// A number in a field.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Coloring(Option<u8>);

impl Coloring {
    pub fn new(c: char) -> Result<Self, String> {
        match c.to_digit(10) {
            Some(0) => Ok(Coloring(None)),
            Some(d) if d <= 9 => Ok(Coloring(Some(d.try_into().unwrap()))),
            Some(d) => Err(format!("Number out of range: {}", d)),
            None => Err(format!("Not a digit: {:?}", c)),
        }
    }
}

impl Default for PlayingField {
    fn default() -> Self {
        PlayingField {
            fields: [Coloring(None); 81],
        }
    }
}

impl fmt::Display for PlayingField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "+===+===+===+===+===+===+===+===+===+")?;
        for (i, line) in self.fields.chunks_exact(9).enumerate() {
            write!(f, "I")?;
            for (j, field) in line.iter().enumerate() {
                let sep = if j % 3 == 2 { "I" } else { "|" };
                if let Some(num) = field.0 {
                    write!(f, "{:^3}{}", num, sep)?;
                } else {
                    write!(f, "   {}", sep)?;
                }
            }
            writeln!(f, "")?;
            if i % 3 == 2 {
                writeln!(f, "+===+===+===+===+===+===+===+===+===+")?;
            } else {
                writeln!(f, "+---+---+---+---+---+---+---+---+---+")?;
            }
        }

        Ok(())
    }
}
