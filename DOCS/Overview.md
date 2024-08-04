# Overview

## Hello World... kinda
I say kinda because right now, we have to use C Embeds (more on C Embeds later), Future support will come
```
_ main :: () {
    @c [printf("hello world");];
}
```

## Comments
Same syntax as python
```
# comment

int x; # another comment
```

## String literals
```
"string literal\n"
@c ['A'];
# There currently aren't character literals but you can use C Embeds or it's numberical value
```

## Variables
The assignment operator is `NOT` the `=` but instead `:`. The equality operator is `=`.<br>
All variables are initalised to zero if not given a value
```
int x;
int y: 0;
# x and y are always equal
```
For more information, please check <a href="./Variables.md">Variables.md</a>

## Constants
Constants are declared with `::`. They must have a value when declared
```
int x :: 10;
x: 15 # ERROR
```

## Numberical literals
When doing math, use `[]`
```
int x: 10;
x: [x + 5];
```

## Loops
There are two loops in Impulse. `loop` and `for`

### Loop
This one acts as a `while` loop or a traditional `for` loop in C.
```
loop (i < 10) [+] {
    # i is type usize here
    # i is out of scope after the closing bracket
}

int i: 0;
loop (i < 10) [+] {

}

int j: 10;
loop (j > 0) [-] {

}

loop (i < 10) {
    i: [i + 2];
}
```
The modifier (`[+]`) is optional. There are 3 modifiers: `[+]`, `[-]`, `[_]`. The plus and minus increment or decrement by one, the void `[_]` is the same as not giving a modifier

### For
Known as the For In or the For Each loop, similar to Zig's syntax
```
@array string fruits: |str.from("apple") str.from("cherry") str.from("banana")|;

for (fruits) [fruit] {
    str.println(fruit);
}

for (fruits) [fruit i] {
    # i is the current index
}
```

The syntax is `for (<elements>) [<elem> <idx>] {}`, the index is optional. Also note that there are only a few types that the for loop supports: `array`, `dyn`, `string`<br>
Notice that there isn't a `,` between `fruit` and `i`. Comma's aren't used to separate expressions in Impulse. They are used as the pipe operator in functional languages

## Branching
### If statement
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

`orif` is the same as `else if` or `elif`. The equality operator is just one `=` since the assigment operator is `:`

## Types
Please check <a href="./Types.md">Types.md</a> as it goes over all the basic types with some slight examples

## Functions
These are similar to C with only a few slight differences
```
_ main :: () {
    
}

string str.from :: (^char word) {
    
}

vec2 vec.new :: (int x int y) {

}
```
Remember, no `,` and need the `::` in the declaration<br>
For more information, please check <a href="./Functions.md">Functions.md</a>.<br>

*more information will be added to the overview eventually, check the documentation as that should be up to date*
