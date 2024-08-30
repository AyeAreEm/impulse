# Branching

## If statement
Similar syntax to C

```
int x: 10;
if (x = 10) {
    # do smth
} orif (x = 9) {
    # do smth
} else {
    # do smth
}
```

`orif` is the same as `else if` or `elif`. The equality operator is just one `=` since the assigment operator is `:`<br>
`or` and `and` are used to chained booleans, works the same as C's `||` and `&&`.

## Switch statement
Similar syntax to C

```
enum Direction :: {
    North;
    East;
    South;
    West;
}

_ main :: () {
    Direction dir: Direction.North;

    switch (dir) {
        case (Direction.North) {
            println("north");
            # this WON'T fall to the next case like C would
        }
        fall (Direction.South) {
            println("south");
            # this WILL fall to the next case
        }
        case {
            println("the rest");
            # not providing an expression acts as the default case
        }
    }
}

```
