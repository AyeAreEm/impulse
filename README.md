# Impulse
## A spur-of-the-moment programming language made to inspire people to bodge something up on impulse.
bodge (v.) - make or repair (something) badly or clumsly.

## Description
### Impulse is made using rust as the transpiler while using gcc as the compiler.
Impulse's main principles are interoperability with C while providing modern niceties, and unique syntax to avoid boredom when programming.<br>
-- Note: Tested on Windows so tread with caution on other OS's --

## Features
- Interopability with C
- Generics
- Strings with length
- Procedural with multi-paradigm features
- Builtin functions
- Manual memory management (no garbage collection)
- Modern standard library
- Zero initalised

```
struct vector :: {
    i32 x;
    i32 y;

    vec new(i32 x i32 y) :: {
        vec vector;
        vector.x: x;
        vector.y: y;
        return vector;
    }
}

_ print_chars(^char word) :: {
    for (word) [ch] {
        @c [printf("%c\n", ch);];
    }
}

_ main() :: {
    vector pos: vec.new(34 35);

    if ([pos.x + pos.y] = 69) {
        @c [printf("haha funny number");];
    }

    loop (i < pos.x) [+] {
        @c [printf("%d\n", i);];
    }

    ^vector pos_ptr: &pos;
    pos_ptr.x: 10;
    pos_ptr.y: 15;
    #      ^ auto dereference

    ^int x: &pos_ptr.x;
    x^: 20;

    print_chars("hello world");
}
```

## Resources
<a href="./Docs/QuickStart.md">Quick Start</a><br>
<a href="./Docs/Overview.md">Overview</a><br>
<a href="./Docs/Docs.md">Docs</a><br>
<a href="./examples">Examples</a>

### Known Bugs
Error when doing math on two functions, only the `-` appears. not sure if the `return` is helping cause this error or it's just something with integer literals and function calls.
```
return [foo(num) - foo(num)];
```

Passing array at index to function parameter
```
foo(bar.data[0]);
```

### Todos / Ideas
```
@def -> define a static label for something (e.g. @def CAP: 100; or @def io: @import "stdio.h";)

defer -> perform action right before the end of current scope

default struct values -> so that there isn't a need for a constructor as its baked into the struct

group -> group of acceptable types. `group number: |u8 i8 i32 int usize|`

any -> any type (may be not type safe)

character literals: the type exists but no way to really make them without C Embed. there are a few options here
    - traditional literal: 'a' 'A' ' '
    - Jai style char literals: @char "a" @char "A" @char " "
    - implicit chars (if length is 0 or 1, it will be a char): "a" "A" " "

standard library:
windows gui -> either natively or a mapping to raylib
hashmaps -> hashmap / hash table / etc. maybe make it fairly efficient and fast
cryptography -> random numbers, hashing, etc
networks -> http 1.0
sockets -> websockets, etc
arenas -> block of memory that all gets freed at the same time
recycle -> used to reuse memory in a arena that should be freed but can't since it's in the block (it will just now be avaible again in the block)
allocators -> different allocators (this also makes sure the dev knows they eventually need to free the memory). need function pointers for this to work
```
