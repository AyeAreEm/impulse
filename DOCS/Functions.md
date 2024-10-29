# Functions
These are similar to Jai or Odin, as well as similar to defining a variable. `<type> <name> :: (<type> <name>) {}`<br>

```
_ main :: () {
    # code
}

# this is in the standard library
@inline _ mem.dealloc :: (^_ block) {
    @c [free(block);];
}
```

## Function Arguments
Note: impulse doesn't use `,` to separate arguments. `(int x int y)` passes two arguments, integer x and integer y.<br>
Arguments are `constant` by default as there isn't a `const` or `var` keyword. This is also to hopefully reduce bugs
```
_ reset :: (int num) {
    num: 10; # this will error since num is constant
}
```

If you want to mutate an argument, use the `@mut` macro to explicitly tell the compiler to treat the argument as a variable

```
_ reset :: (int num) {
    @mut num;
    num: 10;
}
```
