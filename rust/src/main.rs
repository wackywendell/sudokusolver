use std::collections;
use std::env;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::BufRead;

struct Sudoku {
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
    fn name() -> &'static str;
    fn matrix_index(&self, index: usize) -> (usize, usize);
}

struct Row {
    index: usize,
}

impl SubArray for Row {
    fn name() -> &'static str {
        return "R";
    }

    fn matrix_index(&self, index: usize) -> (usize, usize) {
        return (self.index, index);
    }
}

struct Column {
    index: usize,
}

impl SubArray for Column {
    fn name() -> &'static str {
        return "C";
    }

    fn matrix_index(&self, index: usize) -> (usize, usize) {
        return (index, self.index);
    }
}

struct Square {
    index: usize,
}

impl SubArray for Square {
    fn name() -> &'static str {
        return "S";
    }

    fn matrix_index(&self, index: usize) -> (usize, usize) {
        let ii = (self.index - 1) % 3;
        let ij = (self.index - 1) / 3;
        let ji = (index - 1) % 3;
        let jj = (index - 1) / 3;

        return (ii * 3 + ji + 1, ij * 3 + jj + 1);
    }
}

impl Sudoku {
    fn from_reader<R: io::Read>(reader: R) -> io::Result<Sudoku> {
        let mut buf = io::BufReader::new(reader);

        let mut s = Sudoku { rows: [[0; 9]; 9] };

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

    println!("{}", s);
    return Ok(());
}
