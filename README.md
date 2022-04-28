# Generally Underappreciated Algebraic Calculator

`guac` is a minimal but powerful interactive [stack-based](https://en.wikipedia.org/wiki/Reverse_Polish_notation) calculator which displays on just a few lines of the terminal.

## Install

`guac` should work on all modern Linux terminals, and also has been verified to work on Command Prompt and Windows Terminal.

If you have the Rust toolchain installed, simply:

```
$ git clone https://github.com/jacobhenn/guac.git
$ cd guac
$ cargo install --path .
```

If you won't be developing `guac`, run `cargo clean` after installing to save disc space.

## Keybindings

- `q`: **q**uit
- `[e\-.0-9]`: type a number in the input (`e` for e-notation)
- `backspace`
	- if the input is selected and not empty, drop the last char
	- if the input is selected but empty, drop the top of the stack
	- else, drop the expression *to the left of the selection*
- `[\n ]`: push the input to the stack
- `+`: add
- `-`: subtract
- `*`: multiply
- `/`: divide
- `` ` ``: reciprocal
- `~`: opposite (by analogy to Vim's `~`)
- `d`: **d**rop the selected expression, or the topmost expression if the input is selected
- `^`: exponentiate
- `l`: natural **l**og
- `L`: **l**og with given base
- `r`: square **r**oot
- `R`: square
- `%`: modulo
- `;`: toggle the topmost expression's display mode between exact and approximate
- `s`: **s**ine
- `c`: **c**osine
- `t`: **t**angent
- `x`: push **x**
- `h`: move selection to the left (by analogy to Vim's `h`)
- `l`: move selection to the right (by analogy to Vim's `l`)
- `>`: move selected expression to the right (by analogy to Vim's `>>`)
- `<`: move selected expression to the left (by analogy to Vim's `<<`)
- `right`: swap the selected expression (or the topmost one) with the expression to its left
- `a`: cancel selection and jump to input (by analogy to Vim's `A`)
- `v`: enter **v**ariable mode
    - `[A-Za-z]`: type in a custom variable name
    - `esc`: cancel
- `k`: enter **c**onstant mode
    - `p`: **p**i
    - `P`: tau
    - `e`: **e**
    - `c`: speed of light (m·s⁻¹)
    - `G`: **g**ravitational constant (m³·kg⁻¹·s⁻²)
    - `h`: planck constant (J·Hz⁻¹)
    - `H`: reduced planck constant (J·s)
    - `k`: boltzmann constant (J·K⁻¹)
    - `E`: **e**lementary charge (C)
    - `m`: **m**ass of
        - `e`: **e**lectron (kg)
        - `p`: **p**roton (kg)
    - `esc`: cancel
