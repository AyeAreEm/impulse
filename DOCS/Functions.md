# Functions
These are similar to Jai or Odin, as well as similar to defining a variable. `<type> <name> :: (<type+arg> <type+arg>) :: {}`

```
_ main :: () {
    # code
}

# this is in the standard library
@inline _ mem.dealloc :: (^_ block) {
    @c [free(block);];
}
```

# Quirk
The original syntax for functions were a little different but are still available for use in Impulse if it'll help you `grep` it easier. Might be a little hard to read tho
```
_() main :: {
    
}

string(^char) str.from :: {
    
}
```
