# Impulse
## A spur-of-the-moment programming language made to inspire people to bodge something up on impulse.
bodge (v.) - make or repair (something) badly or clumsly.

## Description
### Impulse is made using rust as the translator while using gcc or node as the compiler / runtime.

## Why Impulse
My requirements in a language are fast, simple, fun to write (syntax). Rust is almost perfect but man is it verbose. Odin is pretty much perfect regarding my requirements but
C just feels right, more right than Odin at times. I've been looking into some functional languages because most of them have very interesting syntax but I just find it quite hard to
write `real world code`. That's when I thought, "pffff I could make my own language". I want to make a language that's perfect for me and what I mean by that is I can use it for decent sized projects
as well as the times when I need to whip up something quick and get it working.
Rust is not a whip up something quick language. Python is but that's far too slow, Odin is almost there for me but since it's not as popular as other languages, getting stuck on a problem is torture.
This way with my own language with detailed docs and one day examples, I think I could use impulse to do most things for daily use.

## Features
- Generics
- Strings with length
- Interopability with C
- Simply syntax
- Procedural with functional inspiration
- Decorators (builtin functions)
- Manual memory management (no garbage collection)

## Syntax
-- Note: Take a look at the `hello_world.imp` file to see recently added features in action --<br>

A big inspiration fo Impulse's syntax is C with some Odin and Jai. Simplistic minimalism as I would call it.
The main jist of Impulse is if you're declaring, it goes `<TYPE> <NAME>: <VALUE>`. If you're reading it's `<NAME><TYPE?>: <VALUE>`
<br>
For example,
```
@import "stdio.h";

struct vector: {
    i32 x;
    i32 y;
}

_() main: {
    vector pos;
    pos.x: 34;
    pos.y: 35;

    if ([pos.x + pos.y] = 69) {
        @c [printf("haha funny number");];
    }

    loop (i < pos.x) [+] {
        @c [printf("%d\n", i);];
    }

    vector new_pos;
    new_pos.x: 400;
    new_pos.y: 20;

    ^vector pos_ptr: &pos;
    pos_ptr.x: 10;
    pos_ptr.y: 15;

    ^int x: &pos_ptr.x;
    x^: 20;
}
```

## Installation
-- Note: Only Windows Support Currently (tho it could still work) --

### Requirements
GNU Compiler Collection (gcc) - <a href="https://gcc.gnu.org/install/binaries.html">https://gcc.gnu.org/install/binaries.html</a><br>
Git - <a href="https://git-scm.com/downloads">https://git-scm.com/downloads</a>

### Download Steps
Open a terminal:<br>
`$ cd ~`<br>
`$ mkdir impulse`<br>
`$ cd impulse`<br>
`$ git clone https://github.com/AyeAreEm/impulse.git`<br>
`$ impulse`

### Quick Start
Create `hello.imp`<br>
```
# inside hello.imp
@import "stdio.h";

_() main: {
    @c [printf("hello world");];
}

```
To generate the .exe, run `impulse -b hello.imp hello`<br>
To generate .exe and .c, run `impulse -b -c hello.imp hello`<br>
To generate just .c, run `impulse -c hello.imp hello` (this will generate hello.c)<br><br>

And there you go, your first hello world in impulse... sorta, I know right now you need to use C embedding but eventually this won't be the case 

### Known Errors
Error when doing math on two functions, only the `-` appears. not sure if the `return` is helping cause this error or it's just something with integer literals and function calls.
```
return [foo(num) - foo(num)];
```

Can't call function inside conditions
```
if (foo(num) = 0) {}
```

Passing array at index to function parameter
```
foo(bar.data[0]);
```

### Ideas
```
print -> print something with the type it is
debug -> print something with a specific type

@def -> define a static label for something (e.g. @def CAP: 100; or @def io: @import "stdio.h";)

defer -> perform action right before the end of current scope
sizeof -> returns the size in bytes of a type

default struct values -> so that there isn't a need for a constructor as its baked into the struct
functions in structs -> basically methods

group -> group of acceptable types. `group number: |u8 i8 i32 int usize|`

standard library:
windows gui -> either natively or a mapping to raylib
hashmaps -> hashmap / hash table / etc. maybe make it fairly efficient and fast
networks -> http 1.0
sockets -> websockets, etc
arenas -> block of memory that all gets freed at the same time
recycle -> used to reuse memory in a arena that should be freed but can't since it's in the block (it will just now be avaible again in the block)
allocators -> different allocators (this also makes sure the dev knows they eventually need to free the memory). need function pointers for this to work
```
for more info, look at the <a href="./DOCS/DOCS.md">Docs</a><br>
for examples, check out the <a href="./examples">examples folder</a>
