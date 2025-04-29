use core::fmt;
use std::io::BufRead as _;

mod bitfield_tactics;

fn main() {
    // let zeroed = PlayingField::new(
    //     "000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    // )
    // .unwrap();

    // let mut solved = zeroed.try_field_recursive_solver().unwrap();
    // solved.fields[0] = Number(None);
    // let almost_solved = solved;

    //let almost_solved = PlayingField::new(
    //    "094000130000000000000076002080010000032000000000200060000050400000008007006304008",
    //)
    //.unwrap();
    //println!("{}", almost_solved);

    //let solver = bitfield_tactics::BitfiedTacticsSolver::new(&almost_solved);
    //println!("Solver: {}", solver.extract());
    //let solved = solver.try_solve().unwrap().extract();
    //println!("{}", solved);

    //dbg!(solved.is_solved());

    //solved.print_bad_constraints();
    read_from_stdin();

    return;
}

#[allow(dead_code)]
fn read_from_stdin() {
    let stdin = std::io::stdin();
    let stdin = stdin.lock();

    for (idx, line) in stdin.lines().enumerate() {
        let Ok(line) = line else {
            break;
        };

        let Ok(pf) = PlayingField::new(&line) else {
            continue;
        };

        // Solved
        //let pf = PlayingField::new(
        //    "123456789456789123789123456234567891567891234891234567345678912678912345912345678",
        //)
        //.unwrap();
        // println!("{}", pf);

        // println!("Checked Constraints: {}", pf.check_constraints());
        // println!("Checked Completed: {}", pf.is_complete());
        // println!("Checked Solved: {}", pf.is_solved());

        // for g in group_indices() {
        //     let mut pf = PlayingField::default();
        //     pf.set_group(g, Coloring(Some(8)));
        //     println!("{}", pf);
        // }

        println!("Sudoko {idx}");
        let solver = bitfield_tactics::BitfiedTacticsSolver::new(&pf);
        // println!("Solver: {}", solver.extract());
        let solved = solver.try_solve().unwrap().extract();
        if solved.is_solved() {
            println!("{}", solved);
        } else {
            println!("No solution found");
            return;
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct PlayingField {
    fields: [Number; 9 * 9],
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

fn group_index_index_to_human_readable(i: usize) -> String {
    match i {
        0..=8 => format!("ROW {}", i + 1),
        9..=17 => format!("COL {}", i - 9 + 1),
        18..=26 => format!("SQUARE {}", i - 18 + 1),
        _ => panic!("Invalid group index index {i}"),
    }
}

impl PlayingField {
    pub fn new(f: &str) -> Result<PlayingField, String> {
        let mut fields = vec![];
        for c in f.chars() {
            fields.push(Number::new(c)?);
        }
        Ok(PlayingField {
            fields: fields.try_into().unwrap(),
        })
    }

    pub fn set_group(&mut self, group: [usize; 9], color: Number) {
        for idx in group {
            self.fields[idx] = color;
        }
    }

    pub fn print_bad_constraints(&self) {
        for (i, g) in group_indices().into_iter().enumerate() {
            let mut numbers: Vec<_> = g
                .into_iter()
                .filter_map(|i| self.fields[i].as_number())
                .collect();
            numbers.sort();
            if numbers.windows(2).any(|w| w[0] == w[1]) {
                println!(
                    "Bad Constraint in {}",
                    group_index_index_to_human_readable(i)
                );
            }
        }
    }

    pub fn check_constraints(&self) -> bool {
        for g in group_indices() {
            let mut numbers: Vec<_> = g
                .into_iter()
                .filter_map(|i| self.fields[i].as_number())
                .collect();
            numbers.sort();
            if numbers.windows(2).any(|w| w[0] == w[1]) {
                return false;
            }
        }
        true
    }

    pub fn is_complete(&self) -> bool {
        self.fields.iter().all(|c| c.as_number().is_some())
    }

    pub fn is_solved(&self) -> bool {
        self.is_complete() && self.check_constraints()
    }

    pub fn try_field_recursive_solver(&self) -> Option<Self> {
        let first_unset_index = if let Some(first_unset_index) =
            self.fields.iter().position(|c| c.as_number().is_none())
        {
            first_unset_index
        } else if self.is_solved() {
            return Some(self.clone());
        } else {
            return None;
        };

        let mut new_field = self.clone();
        for i in 1..=9 {
            new_field.fields[first_unset_index] = Number(Some(i));
            if !new_field.check_constraints() {
                continue;
            }
            if new_field.is_solved() {
                return Some(new_field);
            } else if let Some(pf) = new_field.try_field_recursive_solver() {
                return Some(pf);
            }
        }
        None
    }

    pub fn iter_populated_fields(&self) -> impl Iterator<Item = (usize, u8)> + '_ {
        self.fields
            .iter()
            .enumerate()
            .filter_map(|(idx, f)| f.as_number().map(|n| (idx, n)))
    }
}

/// A number in a field.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Number(Option<u8>);

impl Number {
    pub fn new(c: char) -> Result<Self, String> {
        match c.to_digit(10) {
            Some(0) => Ok(Number(None)),
            Some(d) if d <= 9 => Ok(Number(Some(d.try_into().unwrap()))),
            Some(d) => Err(format!("Number out of range: {}", d)),
            None => Err(format!("Not a digit: {:?}", c)),
        }
    }

    pub fn as_number(&self) -> Option<u8> {
        self.0
    }
}

impl Default for PlayingField {
    fn default() -> Self {
        PlayingField {
            fields: [Number(None); 81],
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
