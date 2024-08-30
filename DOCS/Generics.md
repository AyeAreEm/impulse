# Generics
In C, "pseudo" generics or any type is `void*`. This is not that.

```
struct[T] dyn :: {
    ^$T data; # $ declares it of typeid T
    usize len;
    usize cap;

    dyn new :: (typeid T) {
        dyn[T] new;
        new.len: 0;
        new.cap: 32;
        new.data: mem.alloc(T new.cap);
        @c [if (new.data == NULL) exit(1);];

        return new;
    }
}
```

The `$` is used to declare a field or variable of a generic type. If the generic is being passed to a function or used as a subtype, `$` is not used.

# Quirk
Typeid makes a function a C macro, for the most part there is no difference except function parameters<br>
```
# example, not the real function in standard lib
_ push :: (typeid T dyn[T] arr $T data) {
    arr.data[arr.len]: data;
    arr.len: [arr.len + 1];
}

_ main :: () {
    dyn[int] nums: dyn.new(int);
    dyn.push(int nums 10);
}
```
This will cause an error when C expands this, `arr.10[arr.len] = 10;`. Avoid using the same parameter name as a field name.
