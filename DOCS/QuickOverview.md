# Overview

## Hello World... kinda
I say kinda because right now, we have to use C Embeds (more on C Embeds later). Future support will come
```
@import "stdio.h";

_ main(): {
    @c [printf("hello world");];
}
```
Save this as a .imp file. To generate an .exe run `impulse -b hello.imp hello`. If you want to generate a .c file, use `-c` instead of `-b`. If you want both .exe and .c, do `-b -c`.

## Comments
There isn't support for multi-line comments. We use the same syntax as python
```
# This is a comment

int x; # another comment
```

## String literals
These work the same as C. `Note:` we don't currently support character literals, but you can use a C Embed to achieve that.
```
"This is a string\n"
@c ['A'];
```
Using the `string` struct found in the `base` standard lib, you can get the length by simply adding `.len` to the end of a variable;


## Variables
All variables are initalised to zero unless given a value.
```
int x; # same as `int x: 0;`

```
Our assignment operator is `NOT` the `=` but instead `:`.
```
int x: 10;

int y;
y: 15;
```

## Constants
Constants are declared with `::`. They must have a value when declared.
```
int x :: 10;
x: 15; # ERROR
```

## Numberical literals
Impulse supports signed and unsigned integers as well as floats. `Note`: we have a `[]` syntax to do math on numbers.
```
int x: 10;

int y;
y: 15;

y: [y + x];
```

## Loops
There are two kinds of loops in Impulse. The `loop` and `for` loops.<br>
These of course support the usual statements you'd except such as `break` and `continue`.

### Loop
`loop` can work as a while loop or a C `for` (e.g. `for (int i = 0; i < 10; i++) {}`)
```
# the [+] means `i++`
loop (i < 10) [+] {
    @c [printf("%d\n", i);];
}

int x: 10;
loop (x > 0) [-] {}
```
To make this work more like a while loop
```
int x: 0;

# leaving the `[]` lets you specify how to handle the iterations
loop (x < 10) {
    x: [x + 1];
}

bool quit: false;
loop (!quit) {
    # code

    quit: true;
}
```

### For
The `for` loop is the "for in" loop. You'd have seen this in other languages looking like `for fruit in fruits`. Impulse's syntax looks more like Zig's `for` loops.
```
# this `@` will be explained later
@array int nums: |1 2 3 4 5|;

for (nums) [num] {
    @c [printf("%d\n", num);];
}

# to get the current index
for (nums) [num i] {}

```
`Note:` Impulse does not use `,`. The compiler separates by expression or by `<space>`

## Branching
Impulse currently only has support for `if` statements but there will be `switch cases` in the future

## If
```
int x: 10;
if (x = 10) { # since the `=` operator isn't used for assigment, one `=` is used for equality instead of two
    # code
} orif (x = 11) { # same as `elif` from python but in an actual sentence, you're more likely to use the word `or` to describe this situation
    # code
} else {
    # code
}
```

## Functions / Procedures
In Impulse's original design, functions looked like `int(int num) add_five`. While these are still supported, you can do your traditional `int add_five(int num)`
```
# `_` is void
_ main(): {

}

# as mentioned above, this also works
_() main: {

}
```
You might notice the `:` right before the `{`, this is to "assign" this function to this block of code.<br>
This might be changed in the future to `_ main() :: {}` to show that it's a `constant` (newly added).<br>
`Note:` realistically, this does nothing but I think the idea of assigment paints a better mental picture in my head

### Parameters / Arguments
This works like C but remember `no comma's`. `Note:` this idea of no comma's stems from functional language designs
```
int add(int x int y): {
    return [x + y];
}
```
`Note:` currently, function parameters are `mutable` but this might change in the future.

## Types
```
# boolean
bool 

# integers
int i8 i32
usize u8 u32

# character
char

# floats
f32 f64

# types using symbols
^ # pointer
_ # void

# c strings
^char

# strings (with length)
string

# type for types
typeid

# array
@array[10] __subtype__  # e.g. @array[10] int. note, the lenght can be left out

# dynamic array
dyn[__subtype__] # e.g. dyn[int]
```
