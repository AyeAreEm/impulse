# User Definitions

## Struct
Similar syntax to C but we use generics and can create "pseudo" methods. They can't be called off of a variable yet.
```
struct vec2 :: {
    int x;
    int y;

    vec2 new(int x int y) :: {
        vec2 new;
        new.x: x;
        new.y: y;
        return new;
    }
}

struct[T] option :: {
    $T value;
    bool none;
}
```

The syntax is `struct<[<generic>]> <name> :: {}`. Struct fields are declared the same as variable definitions.<br>
Make sure to also check the <a href="./Generics.md">Generics</a> doc for more information

## Enum
These are not tagged unions like in Rust, They are enumerations
Similar syntax to C
```
enum Day :: {
    Monday;
    Tuesday;
    Wednesday;
    Thursday;
    Friday;
    Saturday;
    Sunday;
}

_ main() :: {
    Day today: Day.Monday;
}
```

Enum fields are namespaced with the name of the enum
