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
@import "base/string.imp";

struct vector :: {
    i32 x;
    i32 y;

    vector new :: (i32 x i32 y) {
        vector vec: |x y|;
        return vec;
    }
}

_ print_chars :: (string word) {
    defer { string.dealloc(word); }

    for (word) [ch] {
        println("%c" ch);
    }
}

_ main :: () {
    vector pos: vector.new(34 35);

    if ([pos.x + pos.y] = 69) {
        println("haha funny number");
    }

    loop (i < pos.x) [+] {
        println("%zu" i);
    }

    ^vector pos_ptr: &pos;
    pos_ptr.x: 10;
    pos_ptr.y: 15;
    #      ^ auto dereference

    ^int x: &pos_ptr.x;
    x^: 20;

    print_chars(string.from("hello world"));
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
default struct values -> so that there isn't a need for a constructor as its baked into the struct

group -> group of acceptable types. `group number: |u8 i8 i32 int usize|` (idk what's the name for something like this lmao, it won't be named group tho)

pseudo methods -> struct function that takes a "self" as the first arg, variable can be used as the prefix (e.g. player.update() instead of Player.update(&player))

character literals: the type exists but no way to really make them without C Embed. there are a few options here
    - traditional literal: 'a' 'A' ' '
    - Jai style char literals: @char "a" @char "A" @char " "
    - implicit chars (if length is 0 or 1, it will be a char): "a" "A" " "

constant func args by default -> since there isn't a "const" keyword, there is no way to tell if a function argument should be constant or not, so maybe it should be constant by default. (like in Odin)

variable equal to expression -> similar to how Rust has "let val = if true { 10 } else { 5 };"

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
