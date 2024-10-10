# User Definitions

## Struct
Similar syntax to C but we use generics and can create "pseudo" methods. They can't be called off of a variable yet.
```
struct vec2 :: {
    int x;
    int y;

    vec2 new :: (int x int y) {
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

### Default values
The example from earlier could also be written as this
```
struct vec2 :: {
    int x: 0;
    int y: 10;
}

_ main :: () {
    vec2 a: @default; # this will be 0, 10

    vec2 b: |10|; # this will be 10, 0 because using || overides the default

    vec2 c: @default;
    c.x: 10; # this will be 10, 10

    vec2 d; # this will be 0, 0
}
```

### Macros / Decorators
`struct` has `@shared`.
```
struct vec2 :: @shared {}
```
This makes it a C style union, note: this is not a tagged union. Tagged unions are not currently in Impulse but they will be added in the future

## Enum
These are not tagged unions like in Rust, they are enumerations
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

Enum fields are namespaced with the name of the enum. These have some runtime reflections.

### Field Count
To get the number of fields in an enum, you can do `<namespace>.field_count`. This is a `usize`.<br>
Note: if you plan to continue working in C after using Impulse, you'll have to manually update this variable everytime you add a new enum field
