# Generally Underappreciated Algebraic Calculator

`guac` is a minimal but powerful stack-based (RPN) calculator which displays on just a few lines of the terminal.

## Keybindings

- `q`: **q**uit
- `[0-9.eE]`: type a number in the input (`e` for e-notation, `E` to type `e-`)
- `[\n ]`: push the input to the stack
- `+`: add
- `-`: subtract
- `*`: multiply
- `/`: divide
- `` ` ``: reciprocal
- `~`: opposite (by analogy to Vim's `~`)
- `d`: **d**rop the topmost expression
- `^`: exponentiate
- `l`: natural **l**og
- `L`: **l**og with given base
- `r`: square **r**oot
- `R`: square
- `x`: push **x**
- `%`: modulo
- `s`: **s**ine
- `;`: toggle the topmost expression's display mode between exact and approximate
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

## How does the stack work?

- The central interface to `guac` is the *stack*, a homogenous LIFO collection of *algebraic expressions* (numbers, variables, or functions applied to them).
- The stack is displayed on one line with expressions separated by spaces, so a display like `x^2 3 x` represents a stack with three elements - `x^2`, `3`, and `x`.
- Many keys you can press apply *operations*, like `+` and `/`, to the stack. When you apply an operation to the stack, `guac` pops the required number of arguments from the stack, applies the operation to them, then pushes the result back onto the stack.
- For expample, applying the operations `*` and `+`, in that order, to the example stack above would result in the stack `x^2+3x`.

### Why is the stack so great?

- You don't need to use parentheses.
- For expample, a complicated expression like `((2*x)^(x*7/2))/(((75*x)+3)^2)` can be input with the key sequence `2x*x7⏎2/*^x75*3+2^/` - the asymmetry of postfix notation allows the precedence of operators to be disambiguated by the order in which they appear.
