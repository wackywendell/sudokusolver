import sys


def errprint(*args, **kwargs):
    kw = dict(file=sys.stderr)
    kw.update(kwargs)
    print(*args, **kw)


class SubArray:
    def __init__(self, sudoku):
        self.sudoku = sudoku
    
    def matrix_indices(self, index):
        raise NotImplementedError()
    
    def __getitem__(self, index):
        i, j = self.matrix_indices(index)
        return self.sudoku.row_values[i-1][j-1]
    
    def __setitem__(self, index, value):
        i, j = self.matrix_indices(index)
        self.sudoku.row_values[i-1][j-1] = value
    
    def __len__(self):
        return 9
    
    def __iter__(self):
        for i in range(1, 10):
            yield self[i]
    
    def __str__(self):
        return ''.join(str(v if v else ' ') for v in self)

    def __repr__(self):
        name = type(self).__name__
        return '%s(%s)' % (name[0], str(self))
    
    def is_valid(self):
        ns = set()
        for n in self:
            if n == 0:
                continue
            if n in ns:
                return False
            ns.add(n)
        return True

class Row(SubArray):
    def __init__(self, sudoku, index):
        SubArray.__init__(self, sudoku)
        self.index = index
    
    def matrix_indices(self, index):
        return self.index, index


class Column(SubArray):
    def __init__(self, sudoku, index):
        SubArray.__init__(self, sudoku)
        self.index = index
    
    def matrix_indices(self, index):
        return index, self.index


class Square(SubArray):
    def __init__(self, sudoku, index):
        SubArray.__init__(self, sudoku)
        self.index = index
    
    def matrix_indices(self, index):
        ii, ij = divmod(self.index-1, 3)
        ji, jj = divmod(index-1, 3)
        
        return ii*3 + ji+1, ij*3+jj+1
        

class Sudoku:
    def __init__(self, rows):
        assert len(rows) == 9
        for r in rows:
            assert len(r) == 9
            for v in r:
                assert 0 <= v <= 9
        self.row_values = rows

    def rows(self):
        return [Row(self, index) for index in range(1, 10)]

    def columns(self):
        return [Column(self, index) for index in range(1, 10)]

    def squares(self):
        return [Square(self, index) for index in range(1, 10)]
    
    def copy(self):
        return Sudoku([list(r) for r in self.row_values])
    
    def __getitem__(self, ij):
        i, j = ij
        return self.row_values[i-1][j-1]
    
    def __setitem__(self, ij, value):
        i, j = ij
        assert self.row_values[i-1][j-1] == 0
        self.row_values[i-1][j-1] = value
    
    def subarrays(self, i, j):
        q = ((i-1)//3)*3 + ((j-1)//3) + 1
        return Row(self, i), Column(self, j), Square(self, q)
    
    def fillcount(self):
        return sum([(1 if v > 0 else 0) for r in self.row_values for v in r])
    
    def solved(self):
        return self.fillcount() >= 81

    def __str__(self):
        return '\n'.join(str(r) for r in self.rows())
    
    def is_valid(self):
        assert len(self.row_values) == 9
        for r in self.row_values:
            assert len(r) == 9
            for v in r:
                assert v in range(10)
        rows = [r.is_valid() for r in self.rows()]
        cols = [c.is_valid() for c in self.columns()]
        sqs = [q.is_valid() for q in self.squares()]

        return all(rows) and all(cols) and all(sqs)
    
    @classmethod
    def from_strs(cls, lines):
        rows = []
        for n, line in enumerate(lines, 1):
            zeros = '0- '
            values = [(0 if c in zeros else int(c)) for c in line if c in  zeros + '123456789']

            if len(values) != 9:
                raise Exception(f"Could not interpret line %{n}: {line}")
            rows.append(values)
        
        if len(rows) != 9:
            nr = len(rows)
            raise Exception(f"Only found {nr} rows")
        
        return cls(rows)
    
    def __eq__(self, other):
        return self.row_values == other.row_values
    
    def __hash__(self):
        vs = tuple(sum(self.row_values, []))
        return hash(vs)


class Invalid(Exception): pass


class Solver(Sudoku):
    def __init__(self, sudoku):
        self.sudoku = sudoku

    def fill_subarray(self, subarray):
        filled = 0
        seen = set()
        all_possibilities = {}
        for ix in range(1, 10):
            if subarray[ix] > 0:
                seen.add(subarray[ix])
                continue
            i, j = subarray.matrix_indices(ix)
            row, col, sq = self.sudoku.subarrays(i, j)
            s = set(range(1, 10))
            possibilities = s - set(row) - set(col) - set(sq)
            if len(possibilities) == 0:
                raise Invalid
            if len(possibilities) > 1:
                all_possibilities[ix] = possibilities
                continue
            
            (v,) = possibilities
            filled += 1
            self.sudoku[i, j] = v
            seen.add(v)
        
        for v in range(1, 10):
            if v in seen:
                continue
            
            ixs = [ix for (ix, p) in all_possibilities.items() if v in p]
            if len(ixs) == 0:
                raise Invalid
            
            if len(ixs) > 1:
                continue

            (ix,) = ixs
            filled += 1
            subarray[ix] = v

        return filled    

    def single_pass(self):
        filled = 0
        for r in self.sudoku.rows():
            filled += self.fill_subarray(r)
        
        for c in self.sudoku.columns():
            filled += self.fill_subarray(c)

        for q in self.sudoku.squares():
            filled += self.fill_subarray(q)
        
        return filled
    
    def simple_fill(self):
        filled = 0
        while True:
            n = self.single_pass()
            if n == 0:
                return filled
            filled += n

    def dynamic_solve(self):
        self.simple_fill()
        if not self.sudoku.is_valid():
            raise Invalid
        if self.sudoku.solved():
            return set([self.sudoku])

        all_possibilities = {}
        expand_ixs = None
        expand_possibilities = set()
        for (i, j) in ((i, j) for i in range(1, 10) for j in range(1, 10)):
            if self.sudoku[i, j] > 0:
                continue
            row, col, sq = self.sudoku.subarrays(i, j)
            s = set(range(1, 10))
            possibilities = s - set(row) - set(col) - set(sq)
            if len(possibilities) == 0:
                raise Invalid
            assert len(possibilities) > 1
            if len(possibilities) == 2:
                expand_ixs = i, j
                expand_possibilities = possibilities
                break
            if len(possibilities) > 1:
                all_possibilities[(i, j)] = possibilities
                continue
        
        if not expand_ixs:
            for ij, p in all_possibilities.items():
                if not expand_ixs or len(expand_possibilities) > p:
                    expand_ixs = ij
                    expand_possibilities = p
        
        i, j = expand_ixs
        solutions = set()
        for p in sorted(expand_possibilities):
            new_sudoku = self.sudoku.copy()
            new_sudoku[i, j] = p
            new_solver = Solver(new_sudoku)
            try:
                new_solutions = new_solver.dynamic_solve()
            except Invalid:
                continue

            solutions.update(new_solutions)
        
        return solutions

rows = []
with open(sys.argv[1]) as f:
    lines = [l.strip() for l in f if l.strip()]
    sudoku = Sudoku.from_strs(lines)


solver = Solver(sudoku)

solutions = solver.dynamic_solve()

sep = None
for s in solutions:
    if sep:
        print(sep)
    print(s)
    sep = '-'*9