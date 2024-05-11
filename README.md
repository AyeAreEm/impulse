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
- Non null terminated strings
- Procedural 
- Generics
- Initalise to 0
- Pointers (odin syntax)
- Odin / Jai like syntax
- Decorators (run - run a function immediately after creation)

## Syntax
-- Note: Take a look at the `hello_world.imp` file to see recently added features in action --<br>

A big inspiration fo Impulse's syntax is Odin and Jai with some C. Simplistic minimalism as I would call it.
The main jist of Impulsive is if you're declaring, it goes `<TYPE> <NAME>: <VALUE>`. If you're reading it's `<NAME><TYPE?>: <VALUE>`
<br>
For example,
```
_(string word) talk: {
    print(word);
}

_() main: {
    talk("hello world");
}
```
or
```
_() main: {
    @array int nums: |10 20 30|;
    int new_num: 10;

    nums|0|: nums|0| * new_num;
    nums|1|: nums|1| * new_num;
    nums|2|: nums|2| * new_num;

    print(nums|1|);
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

### Known Errors
Error when doing math on two functions, only the `-` appears. not sure if the `return` is helping cause this error or it's just something with integer literals and function calls.
```
return [foo(num) - foo(num)];
```

### Ideas
```
bool -> boolean

u32 -> 32 bit usigned integer (with support for other lengths of bits)
i32 -> 32 bit integer (with support for other lengths of bits)
f64 -> 64 bit float (with support for 32 bit)

@iter i: 5 {} -> simple loop, not really an iterator type
break -> break out of loop
continue -> continue next to elem in loop

print -> print something with the type it is
debug -> print something with a specific type

@def -> define a static label for something (e.g. @def CAP: 100; or @def io: @import "stdio.h";)

defer -> perform action right before the end of current scope
usize -> unsigned integer type with the same number of bits as the platform's pointer type
sizeof -> returns the size in bytes of a type

default struct values -> so that there isn't a need for a constructor as its baked into the struct
functions in structs -> basically methods

standard library:
windows -> either natively or a mapping to raylib
networks -> http 1.0
sockets -> websockets, etc
arenas -> block of memory that all gets freed at the same time
recycle -> used to reuse memory in a arena that should be freed but can't since it's in the block (it will just now be avaible again in the block)
```
for more info, look at the <a href="./DOCS/DOCS.md">Docs</a><br>
for examples, check out the <a href="./examples">examples folder</a>
