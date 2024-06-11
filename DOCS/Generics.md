# Generics
So in C, there isn't any generics except `void*` but you can "fake" them with macros. Impulse takes this approach to have better interoperability with C (and I was a tad bit lazy to make them how compilers usually do)<br>
Take this bad example:

```
@import "stdio.h";

struct[T] vector: {
    $T x;
    $T y;
}

# the params can't be the same name as the struct fields, you at the c code to see why
vector(typeid T $T x_pos $T y_pos) vector_new: {
    vector[T] new;
    new.x: x_pos;
    new.y: y_pos;
    return new;
}

_() main: {
    vector[i32] pos: vector_new(i32 10 15);

    if (pos.x = 10 and pos.y = 15) {
        @c [printf("the function worked");];
    }
}
```
Let's go through this now.
First you see the `struct[T] vector`, it's a struct that takes a `typeid T` as a value. this is technically handled the same way you'd pass a `typeid` to a function parameter.<br>
You might also be wondering what is the `$` doing. Well `typeid` is a value or variable name inside the struct, not an actual type. To use it as a type, you'd need to preface it with `$`.
This makes more sense when you use a `typeid` inside a function like `vector[T] new;`. You can imagine this like a function call `vector(T)`, it's a variable name not a type.<br>
Going back a bit, you might have noticed the return type on the function is `vector` instead of `vector[T]`, well since `T` is needed when using these vectors at all and `typeid's` are passed as parameters, we use these parameters to distinguish the return type.<br>
This also lets caling the function be a bit less verbose. For example, some languages would do it like this:<br>
`vector<i32> pos = vector_new<i32>(10 15);`

## Side Note
The reason why the function parameters can't be the same as the struct field is because C macros replace all occurences of that identifier with what is passed to the macro (e.g. you pass 10, the code becomes new.10 = 10;)<br>
If you're curious how this looks when it transpiles to C, here it is after making it prettier
```c
#include <stdio.h>
#include <stdint.h>
typedef int32_t i32;

#define vector(T)\
typedef struct {\
    T x;\
    T y;\
} vector_##T;
vector(i32);

#define vector_new(T, x_pos, y_pos) ({\
    vector_##T new = {0};\
    new.x = x_pos;\
    new.y = y_pos;\
    new;\
})

int main() {
    vector_i32 pos = vector_new(i32, 10, 15);

    if (pos.x == 10 && pos.y == 15) {
        printf("the function worked");
    }
}
```
