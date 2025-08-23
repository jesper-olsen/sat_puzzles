# N-Queens

A Rust implementation of the classic **N-Queens problem**, encoded as a **SAT (Boolean satisfiability) problem**.  
The CNF formulas are solved using the [Varisat](https://github.com/jix/varisat) SAT solver.

---

## Installation

Clone the repository and build in release mode:

``` bash
git clone https://github.com/yourusername/N-Queens_sat.git
cd N-Queens_sat
cargo build --release
```

---

## Generate CNF

Generate a .cnf file that can be solved by any SAT solver:

``` bash
cargo run -- generate 5
```

Output
``` text
Generating CNF for 5-Queens problem...
Successfully wrote problem to '5-queens.cnf' (25 variables, 165 clauses)
```

Preview of the CNF file:
``` text
p cnf 25 165
1 2 3 4 5 0
6 7 8 9 10 0
11 12 13 14 15 0
16 17 18 19 20 0
21 22 23 24 25 0
-1 -2 0
-1 -3 0
-1 -4 0
-1 -5 0
```

You can then solve it with e.g. the varisat solver (if you have it in your path):
``` bash
% varisat 5-queens.cnf
```
``` text
c This is varisat 0.2.0-139-g0933fc7
c   release build - rustc 1.88.0 (6b00bc388 2025-06-23)
c Reading file '5-queens.cnf'
c Parsed formula with 25 variables and 165 clauses
s SATISFIABLE
v -1 -2 -3 4 -5 -6 7 -8 -9 -10 -11 -12 -13 -14 15 -16 -17 18 -19 -20 21 -22 -23 -24 -25 0
```

##  Solve N-Queens Directly

Solve and enumerate all the solutions directly:
``` bash
cargo run --release -- solve 5 --all
```
``` text
Solving for 5-Queens...
Found 10 unique solutions for N=5

--- Solution 1 ---
. . . Q .
. Q . . .
. . . . Q
. . Q . .
Q . . . .

--- Solution 2 ---
. . . Q .
Q . . . .
. . Q . .
. . . . Q
. Q . . .

--- Solution 3 ---
. Q . . .
. . . . Q
. . Q . .
Q . . . .
. . . Q .

--- Solution 4 ---
. . . . Q
. Q . . .
. . . Q .
Q . . . .
. . Q . .

--- Solution 5 ---
Q . . . .
. . . Q .
. Q . . .
. . . . Q
. . Q . .

--- Solution 6 ---
. . Q . .
. . . . Q
. Q . . .
. . . Q .
Q . . . .

--- Solution 7 ---
. Q . . .
. . . Q .
Q . . . .
. . Q . .
. . . . Q

--- Solution 8 ---
. . . . Q
. . Q . .
Q . . . .
. . . Q .
. Q . . .

--- Solution 9 ---
Q . . . .
. . Q . .
. . . . Q
. Q . . .
. . . Q .

--- Solution 10 ---
. . Q . .
Q . . . .
. . . Q .
. Q . . .
. . . . Q
```

## License

This project is licensed under the [MIT License](LICENSE).
