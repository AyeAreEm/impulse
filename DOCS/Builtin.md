# Builtin
`@import @c @array @inline`

## Import
`@import "stdlib.h";`<br>
`@import "base/dynamic.imp";`

This can be done for both Header files and Impulse files.<br>
It's good to note that the transpiler will check if the Header file is a libc file and if not, go will put `<name>.c` into the GCC compilation.<br>
If you only have a header file but no C file, you can use `@c [#include "name.h"]`

## C Embed
```
@c [printf("hello world");];
@c [
    if (1 * 1 == 1) {
        printf("huh terrence howard was wrong");
    }
];
```
These are similar to the `__asm__();` in C.<br>
NOTE: these are not checked by the Impulse transpiler, you'll have no type checking and foot guns that are available in C. GCC could catch it but Impulse can't show these errors, you'd have to check the generated C file

## Array
`@array[10] int nums: |1 2 3|;`
`@array int nums: |1 2 3|;`
The syntax is `@array<[length]> <type> <name>`, the length is optional. if not provided, Impulse will count the number of elements given. If none are given, it will error.<br>

NOTE: there is a little quirk with these<br>
In `base/utils.imp`, there is an array struct.
```
struct[T] array: {
    ^$T data;
    usize len;
}
```

The code generated to the C file looks like this (added the newlines), also note that this doesn't work in C++
```
array_int nums = {
    .data = (int[10]){1, 2, 3},
    .len = 10
};
```

## Inline
`@inline int add(int x int y) :: { return [x + y]; }`
This makes a function inlined when compiled by using `static inline __attribute__((always_inline))`
