# Generally Underappreciated Algebraic Calculator

`guac` is a minimal but powerful interactive [stack-based](https://en.wikipedia.org/wiki/Reverse_Polish_notation) calculator which displays on just a few lines of the terminal.

demo (asciinema):
[![asciicast](https://asciinema.org/a/T6CAIqcv5vQayg274QyKYpEMY.png)](https://asciinema.org/a/T6CAIqcv5vQayg274QyKYpEMY)

## selling points

- modal [reverse polish notation](https://en.wikipedia.org/wiki/Reverse_Polish_notation), pretty much the most keystroke-efficient calculator interface you can get
- variables & constants that understand algebra (e.g., `5·π` times `2·π^2` is automatically `10·π^3`)
- seamless input & display in all radices (bases) from 2 to 64 (see the [wiki](https://github.com/jacobhenn/guac/wiki/radices))
- horizontal stack that doesn't display on an alternate terminal screen

## install

`guac` should work on all of [these](https://github.com/crossterm-rs/crossterm#tested-terminals) terminals, and almost certainly more (basically any modern terminal regardless of OS).

### pre-built

download a pre-built executable of the latest [release](https://github.com/jacobhenn/guac/releases).

### compile yourself

if you have the [rust toolchain](https://www.rust-lang.org/tools/install) installed, simply:

```
$ git clone https://github.com/jacobhenn/guac.git
$ cd guac
$ cargo install --path .
```

if you won't be developing `guac`, run `cargo clean` after installing to save disc space.

## keybindings

*see this list in the terminal by running* `guac keys`.

*here, "selected expression" refers to either the manually selected expression, or the topmost expression in the stack (not the input) if none is selected*

- `q` or `escape`: **q**uit
- digit, `.`, or `e`: type a number in the input (`e` for e-notation)
- `#` enter radix mode (see the [wiki](https://github.com/jacobhenn/guac/wiki/radices))
- `backspace`
	- if the input is selected and not empty, drop the last char
	- if the input is selected but empty, drop the top of the stack
	- else, drop the expression *to the left of the selection*
- `enter` or `space`: push the input to the stack
- `+`: add
- `-`: subtract
- `*`: multiply
- `/`: divide
- `` ` ``: reciprocal
- `~`: opposite (by analogy to Vim's `~`)
- `\`: absolute value (by proximity to `|`)
- `d`: **d**rop the selected expression
- `^`: exponentiate
- `g`: natural lo**g**
- `G`: lo**g** with given base
- `r`: square **r**oot
- `R`: square
- `%`: modulo
- `;`: toggle the selected expression's display mode between exact and approximate
- `[`: toggle displaying the selected expression in debug view
- `s`: **s**ine
- `c`: **c**osine
- `t`: **t**angent
- `x`: push **x**
- `h`: select to the left (by analogy to Vim's `h`)
- `l`: select to the right (by analogy to Vim's `l`)
- `>`: move selected expression to the right (by analogy to Vim's `>>`)
- `<`: move selected expression to the left (by analogy to Vim's `<<`)
- `right`: swap the selected expression with the expression to its left
- `a`: cancel selection and jump to input (by analogy to Vim's `A`)
- `ctrl-u`: delete all stack elements to the left of the selection (by convention)
- `:`: enter command mode (by analogy to Vim's `:`) (see the [wiki](https://github.com/jacobhenn/guac/wiki/commands))
- `|`: enter **pipe** mode
    - any char: type a command (to be executed directly, **not** through your `$SHELL`)
    - `enter`: pipe the selected expression to the entered command
    - `escape`: cancel
- `v`: enter **v**ariable mode
    - any char: type in a custom variable name
    - `escape`: cancel
- `k`: enter **c**onstant mode
    - `p`: **p**i
    - `e`: **e**
    - `g`: euler-mascheroni **g**amma constant
    - `c`: **s**peed of light (m·s⁻¹)
    - `G`: **g**ravitational constant (m³·kg⁻¹·s⁻²)
    - `h`: planck constant (J·Hz⁻¹)
    - `H`: reduced planck constant (J·s)
    - `k`: boltzmann **c**onstant (J·K⁻¹)
    - `E`: **e**lementary charge (C)
    - `m`: **m**ass of
        - `e`: **e**lectron (kg)
        - `p`: **p**roton (kg)
    - `escape`: cancel

## known issues

- `guac` doesn't do *too* well with very narrow (<15 column) terminals, or with quickly resizing terminals, although it shouldn't totally break.
- `guac` does not directly set any limit on number size or precision; this is by design. it will absolutely try to perform any operation you tell it to, and will only panic on account of insufficient resources if the `num` crate or any system call it performs does. if it hangs too long on an operation, just run `pkill guac` from another terminal or close the window.
- the algorithms `guac` uses to perform algebra are all hand-written, and their correctness should **not** be assumed at this stage of development. if you encounter an inconsistency, please submit an issue.
- undo/redo is a little janky
