use std::collections;
use std::env;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::iter::FromIterator;

#[derive(Clone)]
pub struct Sudoku {
    rows: [[u8; 9]; 9],
}

impl fmt::Display for Sudoku {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (n, r) in self.rows.iter().enumerate() {
            if n > 0 {
                write!(f, "\n")?;
            }
            for c in r {
                match c {
                    0 => write!(f, "{}", " ")?,
                    _ => write!(f, "{}", c)?,
                }
            }
        }

        return Ok(());
    }
}

trait SubArray {
    fn name(&self) -> &'static str;
    fn matrix_index(&self, index: u8) -> (u8, u8);
}

pub struct Row {
    index: u8,
}

impl SubArray for Row {
    fn name(&self) -> &'static str {
        return "R";
    }

    fn matrix_index(&self, index: u8) -> (u8, u8) {
        return (self.index, index);
    }
}

pub struct Column {
    index: u8,
}

impl SubArray for Column {
    fn name(&self) -> &'static str {
        return "C";
    }

    fn matrix_index(&self, index: u8) -> (u8, u8) {
        return (index, self.index);
    }
}

pub struct Square {
    index: u8,
}

impl SubArray for Square {
    fn name(&self) -> &'static str {
        return "S";
    }

    fn matrix_index(&self, index: u8) -> (u8, u8) {
        let ii = (self.index - 1) / 3;
        let ij = (self.index - 1) % 3;
        let ji = (index - 1) / 3;
        let jj = (index - 1) % 3;

        return (ii * 3 + ji + 1, ij * 3 + jj + 1);
    }
}

impl Sudoku {
    fn empty() -> Sudoku {
        Sudoku { rows: [[0; 9]; 9] }
    }

    fn from_reader<R: io::Read>(reader: R) -> io::Result<Sudoku> {
        let mut buf = io::BufReader::new(reader);

        let mut s = Sudoku::empty();

        let mut line = String::new();

        let mut rix = 0;
        let mut cix = 0;

        loop {
            match buf.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {}
                Err(e) => {
                    return Err(e);
                }
            }
            let trimmed = line.trim();

            for c in trimmed.chars() {
                let n = match c {
                    '1'...'9' => c.to_digit(10).expect("This shouldn't happen"),
                    '0' | '-' | 'x' => 0,
                    _ => continue,
                };

                if cix >= 9 || rix >= 9 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Too many characters",
                    ));
                }
                s.rows[rix][cix] = n as u8;
                cix += 1;
            }

            if cix < 9 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Too few characters",
                ));
            }

            rix += 1;
            cix = 0;
            line.clear();
        }

        if rix < 9 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Too few rows",
            ));
        }

        return Ok(s);
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Cell {
    Value(u8),
    Possibilities(collections::BTreeSet<u8>),
}

enum Removal {
    Fixed(u8),
    Removed,
    NotFound,
}

impl Cell {
    fn remove_possibility(&mut self, value: u8) -> Removal {
        let ps = match self {
            Cell::Value(_) => return Removal::NotFound,
            Cell::Possibilities(p) => p,
        };
        if !ps.contains(&value) {
            return Removal::NotFound;
        }
        ps.remove(&value);
        if ps.len() == 1 {
            let &v = ps.iter().next().unwrap();
            *self = Cell::Value(v);
            return Removal::Fixed(v);
        }
        return Removal::Removed;
    }
}

impl Default for Cell {
    fn default() -> Cell {
        return Cell::Possibilities(collections::BTreeSet::from_iter(1..10));
    }
}

#[derive(Clone)]
pub struct UnsolvedSudoku {
    rows: [[Cell; 9]; 9],
}

#[derive(Clone, Copy)]
enum PossibleLocations {
    None,
    Filled,
    Single(u8, u8),
    Many,
}

impl UnsolvedSudoku {
    pub fn empty() -> UnsolvedSudoku {
        return UnsolvedSudoku {
            rows: Default::default(),
        };
    }

    fn get_subarrays(rix: u8, cix: u8) -> (Row, Column, Square) {
        let qix = ((rix - 1) / 3) * 3 + ((cix - 1) / 3) + 1;
        return (
            Row { index: rix },
            Column { index: cix },
            Square { index: qix },
        );
    }

    pub fn get(&self, rix: u8, cix: u8) -> &Cell {
        return &self.rows[(rix - 1) as usize][(cix - 1) as usize];
    }

    pub fn get_mut(&mut self, rix: u8, cix: u8) -> &mut Cell {
        return &mut self.rows[(rix - 1) as usize][(cix - 1) as usize];
    }

    // Set the value at (rix, cix) with value, and recursively remove that possibility
    // from all cells in the same row, column, and square.
    fn set(&mut self, rix: u8, cix: u8, value: u8) {
        self.rows[(rix - 1) as usize][(cix - 1) as usize] = Cell::Value(value);
        let (r, c, s) = UnsolvedSudoku::get_subarrays(rix, cix);
        let subarrays = &[
            Box::new(r) as Box<SubArray>,
            Box::new(c) as Box<SubArray>,
            Box::new(s) as Box<SubArray>,
        ];

        for ix in 1..10 {
            for a in subarrays {
                let (i, j) = a.matrix_index(ix);
                match self.get_mut(i, j).remove_possibility(value) {
                    Removal::Fixed(w) => self.set(i, j, w),
                    _ => continue,
                }
            }
        }
    }

    fn simple_solve(&mut self) {
        let mut filled = -1;
        while filled != 0 {
            filled = 0;
            for ix in 1..10 {
                let subarrays = &[
                    Box::new(Row { index: ix }) as Box<SubArray>,
                    Box::new(Column { index: ix }) as Box<SubArray>,
                    Box::new(Square { index: ix }) as Box<SubArray>,
                ];
                for a in subarrays {
                    let mut locs = [PossibleLocations::None; 9];
                    for inner in 1..10u8 {
                        let (rix, cix) = a.matrix_index(inner);
                        let ps = match self.get(rix, cix) {
                            &Cell::Value(v) => {
                                locs[(v - 1) as usize] = PossibleLocations::Filled;
                                continue;
                            }
                            &Cell::Possibilities(ref ps) => ps,
                        };

                        for p in ps {
                            locs[(p - 1) as usize] = match locs[(p - 1) as usize] {
                                PossibleLocations::None => PossibleLocations::Single(rix, cix),
                                PossibleLocations::Single(..) => PossibleLocations::Many,
                                _ => continue,
                            }
                        }
                    }

                    for (ix, p) in locs.iter().enumerate() {
                        let v = (ix + 1) as u8;
                        match p {
                            &PossibleLocations::Single(rix, cix) => {
                                self.set(rix, cix, v);
                                filled += 1;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

impl From<Sudoku> for UnsolvedSudoku {
    fn from(s: Sudoku) -> UnsolvedSudoku {
        let mut u = UnsolvedSudoku::empty();
        for (rix, row) in s.rows.iter().enumerate() {
            for (cix, &v) in row.iter().enumerate() {
                if v != 0 {
                    u.set((rix + 1) as u8, (cix + 1) as u8, v);
                    continue;
                }
            }
        }
        return u;
    }
}

impl Into<Sudoku> for UnsolvedSudoku {
    fn into(self) -> Sudoku {
        let mut s = Sudoku::empty();
        for (rix, row) in self.rows.iter().enumerate() {
            for (cix, cell) in row.iter().enumerate() {
                s.rows[rix][cix] = match cell {
                    &Cell::Value(v) => v,
                    &Cell::Possibilities(_) => 0,
                };
            }
        }
        return s;
    }
}

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();
    let filename = match args.as_slice() {
        &[_, ref f] => f,
        _ => {
            println!("Usage: sudokusolver file");
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Needs one input.",
            ));
        }
    };

    let f = File::open(filename).expect("file not found");

    let s = Sudoku::from_reader(f)?;
    let mut u: UnsolvedSudoku = s.into();
    u.simple_solve();
    let solved: Sudoku = u.into();

    println!("{}", solved);
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rows() {
        for i in 1..10 {
            let r = Row { index: i };
            for j in 1..10 {
                let (rix, cix) = r.matrix_index(j);
                assert_eq!(rix, i);
                assert_eq!(cix, j);

                let (r2, _, _) = UnsolvedSudoku::get_subarrays(i, j);

                assert_eq!(rix, r2.index);
            }
        }
    }

    #[test]
    fn test_squares() {
        for i in 1..10 {
            let q = Square { index: i };
            for j in 1..10 {
                let (rix, cix) = q.matrix_index(j);
                let (_, _, q2) = UnsolvedSudoku::get_subarrays(rix, cix);
                println!(
                    "Square ({}, {}) -> ({}, {}) -> {}",
                    i, j, rix, cix, q2.index
                );
                assert_eq!(i, q2.index);
            }
        }
    }
}
