# Types
Impulse has the basics. we plan to implement more types such as booleans, pointers & addresses, floats, etc
<br>
```
_ -> void

u8 -> 8 bit unsigned integer
i8 -> 8 bit signed integer
char -> 8 bit integer to interop with C (OS dependant if signed or unsigned)
i32 -> 32 bit signed integer
int -> OS dependant sized signed integer, same as C
usize -> OS dependant sized unsigned integer, big enough for the OS's pointer type

bool -> boolean
^ -> pointer (also dereference operator when put behind the variable name)
typeid -> type (to pass a type as a value, goes in hand mainly with generics)

@array[10] (type) -> an array of type with optional length (optional if initalised with values), e.g. @array int
```

<a href="./Variables.md">Next -> Variables</a>
