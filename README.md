# Generally Underappreciated Algebraic Calculator

`guac` is a minimal but powerful interactive [stack-based](https://en.wikipedia.org/wiki/Reverse_Polish_notation) calculator which displays on just a few lines of the terminal.

```
$ guac
x sqrt(π)/5 78█
                                                                          (q: quit) rad
```

## install

`guac` should work on all of [these](https://github.com/crossterm-rs/crossterm#tested-terminals) terminals, and probably more. if you have the [rust toolchain](https://www.rust-lang.org/tools/install) installed, simply:

```
$ git clone https://github.com/jacobhenn/guac.git
$ cd guac
$ cargo install --path .
```

if you won't be developing `guac`, run `cargo clean` after installing to save disc space.

## keybindings

*see this list in the terminal by running* `guac keys`.

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
- `\`: absolute value (by proximity to `|`)
- `d`: **d**rop the selected expression, or the topmost expression if the input is selected
- `^`: exponentiate
- `l`: natural **l**og
- `L`: **l**og with given base
- `r`: square **r**oot
- `R`: square
- `%`: modulo
- `;`: toggle the selected (or topmost) expression's display mode between exact and approximate
- `s`: **s**ine
- `c`: **c**osine
- `t`: **t**angent
- `x`: push **x**
- `h`: select to the left (by analogy to Vim's `h`)
- `l`: select to the right (by analogy to Vim's `l`)
- `>`: move selected expression to the right (by analogy to Vim's `>>`)
- `<`: move selected expression to the left (by analogy to Vim's `<<`)
- `right`: swap the selected expression (or the topmost one) with the expression to its left
- `a`: cancel selection and jump to input (by analogy to Vim's `A`)
- `|`: enter **pipe** mode
    - `any char`: type a command (to be executed directly, **not** through your `$SHELL`)
    - `enter`: pipe the selected expression (or the topmost one) to the entered command
    - `esc`: cancel
- `v`: enter **v**ariable mode
    - `[A-Za-z]`: type in a custom variable name
    - `esc`: cancel
- `k`: enter **c**onstant mode
    - `p`: **p**i
    - `P`: tau
    - `e`: **e**
    - `g`: euler-mascheroni constant
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

## known issues

- `guac` doesn't do *too* well with very narrow (⪅15 column) terminals, or with quickly resizing terminals, although it won't totally break.
- `guac` does not directly set any limit on number size or precision; this is by design. it will absolutely try to perform any operation you tell it to, and will only panic on account of insufficient resources if the `num` crate or any system call it performs does. if it hangs too long on an operation, just run `pkill guac` from another terminal or close the window.
- the algorithms `guac` uses to perform algebra are all hand-written, and their correctness has not been proven. if you encounter an inconsistency, please submit an issue.
