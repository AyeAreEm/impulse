# C Interopability / Embedding
If you want to be able to access the C standard library and write in C, you just import the header file and use the C embed.
for example:
```
@import "stdio.h"

_() main: {
    @c [
        printf("hello world");
    ];

    string greet: "hello world";
    @c [
        printf("%s", greet.data);
    ];
}
```

This also works with header files not from the C standard library, just make sure you have the C file as well.

<a href="./Generics.md">Next -> Generics</a>
