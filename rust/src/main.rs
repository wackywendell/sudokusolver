use std::env;
use std::fs::File;
use std::io;
use std::io::BufRead;

struct Sudoku {
    rows: [[u8; 9]; 9],
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

    for row in s.rows.iter() {
        println!(
            "{}{}{}{}{}{}{}{}{}",
            row[0], row[1], row[2], row[3], row[4], row[5], row[6], row[7], row[8]
        )
    }

    println!("{:?}", args);
    return Ok(());
}
