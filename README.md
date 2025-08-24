# SAT Puzzle Solvers in Rust
A collection of tools that explores classic logic puzzles by encoding them as Boolean Satisfiability (SAT) problems.

This project takes a declarative approach: the rules of each puzzle are described as a set of logical constraints, which are then compiled into a Conjunctive Normal Form (CNF) formula. A generic SAT solver is then leveraged to find valid solutions.

The core solver logic uses the [Varisat](https://github.com/jix/varisat) library.

## Supported Puzzles
N-Queens: Place N queens on an N×N chessboard so that no two queens threaten each other.
Sudoku: Fill a 9×9 grid with digits so that each column, each row, and each of the nine 3×3 subgrids contain all of the digits from 1 to 9.
Minesweeper: (Coming Soon) Deduce the location of hidden mines in a grid based on numeric clues.

## Features
CNF Generation: Creates standard DIMACS .cnf files for various puzzles, compatible with most external SAT solvers.
Direct Solving: Solves puzzles directly in-memory without creating an intermediate file.
Solution Enumeration: Finds and counts all unique solutions for a given puzzle configuration.
Visualization: Renders found solutions in an easy-to-read, puzzle-specific format.

## Getting Started

### Prerequisites

You need to have the Rust toolchain installed. If you don't, get it at [rustup.rs.](https://rustup.rs/)

---

## Getting Started

## Prerequisites

You need to have the Rust toolchain installed. If you don't, get it at [rustup.rs.](https://rustup.rs/)

## Installation

Clone the repository and build in release mode:

``` bash
git clone https://github.com/yourusername/N-Queens_sat.git
cd N-Queens_sat
cargo build --release
```
The executable will be located at target/release/nqueens_sat.

---

## Usage

The tool provides two main modes of operation:
* generate - write the CNF formula to a file, for use with external SAT solvers.
* solve - solve the problem directly in-memory using Varisat and visualize solutions.

## 1. Generate CNF

This command creates a .cnf file that describes the constraints for an N-Queens problem. This file can be used with external SAT solvers.

``` bash
cargo run --release -- generate 8
```

Output
``` text
Generating CNF for 8-Queens problem...
Successfully wrote problem to '8-queens.cnf' (64 variables, 736 clauses)
```

Preview of the CNF file:
``` text
p cnf 64 736
1 2 3 4 5 6 7 8 0
9 10 11 12 13 14 15 16 0
17 18 19 20 21 22 23 24 0
25 26 27 28 29 30 31 32 0
33 34 35 36 37 38 39 40 0
41 42 43 44 45 46 47 48 0
49 50 51 52 53 54 55 56 0
57 58 59 60 61 62 63 64 0
-1 -2 0
```

You can then solve this file with varisat or another solver:
``` bash
% varisat 8-queens.cnf
```

Solver output:
``` text
c This is varisat 0.2.0-139-g0933fc7
c   release build - rustc 1.88.0 (6b00bc388 2025-06-23)
c Reading file '8-queens.cnf'
c Parsed formula with 64 variables and 736 clauses
s SATISFIABLE
v -1 -2 -3 -4 5 -6 -7 -8 -9 10 -11 -12 -13 -14 -15 -16 -17 -18 -19 20 -21 -22 -23 -24 -25 -26 -27 -28 -29 -30 31 -32 -33 -34 35 -36 -37 -38 -39 -40 -41 -42 -43 -44 -45 -46 -47 48 -49 -50 -51 -52 -53 54 -55 -56 57 -58 -59 -60 -61 -62 -63 -64 0
```

## 2. Solve and Visualize Directly

This command solves the problem in-memory and prints the solutions to the console.

Find a single solution:
``` bash
cargo run --release -- solve 8
```
``` text
Solving for 8-Queens...
Found a solution for N=8

--- Solution 1 ---
. . . . Q . . .
. Q . . . . . .
. . . Q . . . .
. . . . . . Q .
. . Q . . . . .
. . . . . . . Q
. . . . . Q . .
Q . . . . . . .
```

Find and count *all* solutions:
``` bash
cargo run --release -- solve 8 --all
```
``` text
Solving for 8-Queens...
Found 92 unique solutions for N=8

--- Solution 1 ---
. . . . Q . . .
. Q . . . . . .
. . . Q . . . .
. . . . . . Q .
. . Q . . . . .
. . . . . . . Q
. . . . . Q . .
Q . . . . . . .

--- Solution 2 ---
. Q . . . . . .
. . . Q . . . .
. . . . . Q . .
. . . . . . . Q
. . Q . . . . .
Q . . . . . . .
. . . . . . Q .
. . . . Q . . .

...
```

## How It Works: SAT Encoding
Internally, each puzzle is encoded as a Boolean satisfiability formula. The puzzle's state is mapped to a set of Boolean variables, and its rules are expressed as logical constraints (clauses).

Example: N-Queens Encoding

Variables: A boolean variable x_r,c is created for each square (r, c) on the board. x_r,c is true if a queen is on that square, and false otherwise. For an 8x8 board, this means 64 variables.

Constraints: The rules of chess are encoded as logical clauses:

1. At least one queen per row: For each row r, the clause (x_r,1 OR x_r,2 OR ... OR x_r,N) must be true.

2. At most one queen per row: For each row r and each pair of columns c1, c2, the clause (NOT x_r,c1 OR NOT x_r,c2) must be true.

3. At most one queen per column: Same logic, but for columns.

4. At most one queen per diagonal: Same logic, but for all pairs of squares on the same diagonal.

The combination of all these clauses forms the final CNF formula that the SAT solver is given.


## License

This project is licensed under the [MIT License](LICENSE).
