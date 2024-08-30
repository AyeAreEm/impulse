# Overview

## Hello World
```
_ main :: () {
    println("hello world");
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
'A'
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
When doing math, use `[]`. This is similar to Gleam where it uses `{}` around math operations
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
    # the [+] is the same as i++
}

int i: 0;
loop (i < 10) [+] {

}

int j: 10;
loop (j > 0) [-] {
    # [-] is the same as i--
}

loop (i < 10) {
    i: [i + 2];
}
```
The modifier (`[ ]`) is optional. There are 3 modifiers: `[+]`, `[-]`, `[_]`. The plus and minus increment or decrement by one, the void `[_]` is the same as not giving a modifier

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

The syntax is `for (<elements>) [<elem> <idx>] {}`, the index is optional.<br>
Also note that there are only a few types that the for loop supports: `array`, `dyn`, `string`, `str`, `^char`<br>
Notice that there isn't a `,` between `fruit` and `i`. Comma's aren't used to separate expressions in Impulse. They will be used as the pipe operator in functional languages (not implemented yet)

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

### Switch statement
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
There's a few keywords that come with this. `case` is the keyword when you want only one this particular code block to run. `fall` is when you want to run this code block and fall through to the one below it like in C<br>

## Types
Please check <a href="./Types.md">Types.md</a> as it goes over all the basic types with some slight examples

## Functions
Jai and Odin were big inspirations for this language and you can tell here. the syntax is `<type> <name> :: (<type> <name>) {}`
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

### Generic Functions
Remember that Impulse transpiles to C and one of the aims of this language is to have top tier interop with C. So Impulse has to do something *weird* to get this working between the two<br>
Here's an example from `base/dynamic.imp`

```
_ push :: (typeid T dyn[T] arr $T elem) {
    if ([arr.len + 1] >= arr.cap) {
        arr.cap: [[arr.cap + arr.len + 1] * 2];
        usize T_size: size_of(T);
        arr.data: mem.realloc(arr.data [arr.cap * T_size]);
    }

    arr.data[arr.len]: elem;
    arr.len: [arr.len + 1];
}
```
`typeid` is a way to pass a type to a function.<br>
We then can use it in two different ways: `to pass` or `to use`. You can see the `$`, that is `to use`. Whenever you want to declare a variable with this type, you use a `$T` before the variable name.<br>
`to pass`, there is no need to use a `$`. for example the line `usize T_size: size_of(T);`<br>

The way we get this to work in C is macros (Impulse will type check beforehand for arr and elem). That code transpiles to
```c
#define dyn__push(T, arr, elem) ({\
    if (arr.len + 1 >= arr.cap) {\
        arr.cap = (arr.cap + arr.len + 1) * 2;\
        usize T_size = size_of(T);\
        arr.data = mem__realloc(arr.data, arr.cap * T_size);\
    }\
    arr.data[arr.len] = elem;\
    arr.len = arr.len + 1;\
})
```
Please check <a href="./Generics.md">Generics.md</a> as there's more information in there as well as an example with structs

## User Definitions (Structs and Enums)
### Structs
These are similar to C, Jai and Odin
```
struct vec2 :: {
    f64 x;
    f64 y;
}
```
Currently, Impulse does not support default values in structs but there are plans to add them.<br>
We do support functions inside structs which provide a namespace for these functions. In the future, there will also be methods. Overall these work similar to Zig

```
struct vec2 :: {
    f64 x;
    f64 y;

    vec2 new :: (f64 x f64 y) {
        vec2 new: |x y|;
        return new;
    }

    _ add :: (^vec2 v f64 n) {
        v.x: [v.x + n];
        v.y: [v.y + n];
    }
}

_ main :: () {
    vec2 pos: vec2.new(10 15);
    vec2.add(&pos 10); # this will be pos.add(10); in the future, currently unsupported
}
```

## Enums
These work exactly like C's enums. These are not Rust enums. They share similar syntax to Jai
```
enum Direction :: {
    North;
    East :: 10;
    South; # this will now be 11
    West;
}
```

Please check <a href="./UserDefs.md">UserDefs.md</a> for more information<br>

*more information will be added to the overview eventually, check the documentation as that should be up to date*
