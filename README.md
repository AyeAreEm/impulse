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

Making a new type with a generic struct
```
typeid vec2 :: array[f32];
```

### Todos / Ideas
#### Default struct values
This could replace constructors or the `new` / `init` functions.
```
struct Player :: {
    Vector2f32 pos: |0 0|:
    i32 health: 100;
}
```

#### Grouped Types
Group of acceptables types. This could be similar to how Go does generics
```
typeid number: |u8 i8 u16 i16 u32 i32 u64 i64 uint int usize|;

number add :: (number x number y) {
    return [x + y];
}
```

#### Pseudo methods
Struct function that takes a `self` as the first arg, variable can be used as the prefix.
```
_ string.push_char :: (^string word char ch) {
    # implementation
}

string word: string.from("hello");
word.push_char('!');
```

#### Constant function arguments by default
Since there isn't a `const` keyword, there is no way to tell if an arg is constant or not. so maybe it should be constant by default. (like in Odin)

#### Constant function argument syntax alternative to above
```
i32 add :: (i32 :: x i32 :: y) {
    x: 10; # error, constant reassignment
    return [x + y];
}
```

#### Variable equal to expression
Not sure on the syntax.
```
bool truth: true;
int x: if (truth) { 10; } else { 5; };
```

#### Tagged unions
This most likely won't be a C tagged union where it's a struct with an enum and a union. Might use a `void* data;` as the union.
```
union Token :: {
    Plus;
    Minus;
    Divide;
    Multiply;

    i32 Number;
}

Token t: union.new(Token Token.Number 10);
switch (t) {
    case (Token.Number) [num] {
        println("number is %d" num):
    }
    case (Token.Plus) {
        println("+");
    }
    case {
        println("other symbols");
    }
}
```


#### Standard library
```
windows gui -> either natively or a mapping to raylib
hashmaps -> hashmap / hash table / etc. maybe make it fairly efficient and fast
cryptography -> random numbers, hashing, etc
networks -> http 1.0
sockets -> websockets, etc
arenas -> block of memory that all gets freed at the same time
recycle -> used to reuse memory in a arena that should be freed but can't since it's in the block (it will just now be avaible again in the block)
allocators -> different allocators (this also makes sure the dev knows they eventually need to free the memory). need function pointers for this to work
```
