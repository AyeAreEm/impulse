# Impulse
## A spur-of-the-moment programming language made to inspire people to bodge something up on impulse.
bodge (v.) - make or repair (something) badly or clumsly.

## Description
### Impulse is made using rust as the translator while using gcc or node as the compiler / runtime.

## Why Impulse
I like rust but man is it verbose. I have been writing a lot of Odin recently and it's amazing but I feel like it's not my perfect language.
But when I say perfect, I mean the language I can use for decent sized projects as well as the times when I need to whip up something quick and get it working.
Rust is not a whip up something quick language. Python is but that's far too slow, Odin is almost there for me but since it's not as popular as other languages, getting stuck on a problem is torture.
This way with my own language with detailed docs and one day examples, I think I could use impulse to do most things for daily use.

## Features
- Non null terminated strings
- Structs
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
    print(word)
}

_() main: {
    talk("hello world")
}
```
or
```
_() main: {
    @array int nums: |10 20 30|
    int new_num: 10

    nums|0|: nums|0| * new_num
    nums|1|: nums|1| * new_num
    nums|2|: nums|2| * new_num

    print(nums|1|)
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

### Ideas
```
bool -> boolean
f64 -> 64 bit float
@iter i: 5 {} -> simple loop, not really an iterator type
loop (i < 5) [+] {} -> actual loop

@import "base/io.imp" -> library of imp files
@import "libc/stdio.h" -> library of c files
```
for more info, look at the <a href="./DOCS/DOCS.md">Docs</a>
