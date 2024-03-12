# Impulse
## A spur-of-the-moment programming language made to inspire people to bodge something up on impulse.
bodge (v.) - make or repair (something) badly or clumsly.

## Description
### Impulse is made using rust as the compiler but will compile to an .exe file using c.

## Features
- Non null terminated strings
- Structs
- Procedural with OOP elements
- Pointers (odin syntax)
- Odin / Jai like syntax

## Syntax
A big inspiration fo Impulse's syntax is Odin and Jai with some C. Simplistic minimalism as I would call it.
The main jist of Impulsive is if you're declaring, it goes `<TYPE> <NAME>: <VALUE>`. If you're reading it's `<NAME><TYPE?>: <VALUE>`
<br>
For example,
```
_(string word) talk: {
    print(word)
}

_() main: {
    talk(word)
}
```
or
```
_() main: {
    []int nums: [10, 20, 30]

    nums[1]: nums[1] * 2

    print(nums[1])
}
```
for more info, look at the <a href="./DOCS.md">Docs</a>
