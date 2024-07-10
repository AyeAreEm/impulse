# Types
Impulse has the basics. we plan to implement more types such as booleans, pointers & addresses, floats, etc
<br>
```
_ -> void

u8 -> 8 bit unsigned integer
i8 -> 8 bit signed integer
char -> 8 bit integer to interop with C (OS dependant if signed or unsigned)
u32 -> 32 bit unsigned integer
i32 -> 32 bit signed integer
int -> OS dependant sized signed integer, same as C
usize -> OS dependant sized unsigned integer, big enough for the OS's pointer type

bool -> boolean
^ -> pointer (also dereference operator when put behind the variable name)
typeid -> type (to pass a type as a value, goes in hand mainly with generics)

@array[10] (type) -> an array of type with optional length (optional if initalised with values), e.g. @array int
```

## Note about arrays
`@array` is a bit of a weird compiler hack<br>
In the standard lib, there's a struct definition
```
struct[T] array: {
    ^$T data;
    usize len;
}
```
before arrays in Impulse were the same as in C, just a pointer. I want arrays in Impulse to also carry the length as well
so something like
```
@array int nums: |1 2 3 4 5|;
```
transcompiles as (not that this does not work in C++)
```
array_int nums = {.data = (int[]){1, 2, 3, 4, 5}, .len = 5};
```

<a href="./Variables.md">Next -> Variables</a>
