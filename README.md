# Sudoku Solver

This is a straightforward sudoku solver, made for my own amusement. It applies simple rules, and if those don't fill the whole puzzle, it alternates brute force (i.e. guessing) and the simple rules until it does arrive at a solution.

Error-handling is minimal; again, this was made for my own amusement.

## Usage

`python solver.py input.txt`

The input file must contain 9 lines, with 9 characters (plus newline) each. Those 9 characters should be either 1-9, or "0", "-", or " " for unfilled.

See examples in `moderate.txt` and `challenge.txt`.

## How it Works

The solver first fills as many cells as it can with the "simple" filler. If that does not fully solve the puzzle, it then attempts the "dynamic" solution.

### Simple Filling

- For each cell, find all values in its row, column and box; if only one value remains to that cell, use it.
- For each value 1-9 and each row, column, and square, see if how many cells in that row/column/square could take that value. If there is exactly one, fill that cell with the value.

### Dynamic Filling

If the above does not provide a full solution, it attempts a "dynamic" solution:

- Fill using the "simple" solution. If that produces a full solution, return it.
- Find the first cell that has the fewest possibilities still open to it. Make a copy of the puzzle for each of those possibilities, fill that value into the cell, and attempt to solve dynamically again.