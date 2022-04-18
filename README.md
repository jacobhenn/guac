# Generally Underappreciated Algebraic Calculator

`guac` is a minimal but powerful stack-based (RPN) calculator which displays on just a few lines of the terminal.

## Keybindings

- `d`: **d**rop the topmost expression
- `q`: **q**uit
- `` ` ``: toggle the topmost expression's display mode between approximate and exact (precision is not lost when displaying approximates)
- `x`: push **x**
- `+`: add
- `-`: subtract
- `*`: multiply
- `/`: divide
- `^`: exponentiate

## How does the stack work?

- The central interface to `guac` is the *stack*, a homogenous LIFO collection of *algebraic expressions* (numbers, variables, or functions applied to them).
- The stack is displayed on one line with expressions separated by spaces, so a display like `x^2 3 x` represents a stack with three elements - `x^2`, `3`, and `x`.
- Many keys you can press apply *operations*, like `+` and `/`, to the stack. When you apply an operation to the stack, `guac` pops the required number of arguments from the stack, applies the operation to them, then pushes the result back onto the stack.
- For expample, applying the operations `*` and `+`, in that order, to the example stack above would result in the stack `x^2+3x`.

### Why is the stack so great?

- You don't need to use parentheses.
- For expample, a complicated expression like `((2*x)^(x*7/2))/(((75*x)+3)^2)` can be input with the key sequence `2x*x7⏎2/*^x75*3+2^/` - the asymmetry of postfix notation allows the precedence of operators to be disambiguated by the order in which they appear.
