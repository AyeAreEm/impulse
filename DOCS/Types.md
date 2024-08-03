# Primitives
## Void
`_` is void but it's also used as a general symbol to mean "Nothing".

## Integers
### Unsigned
`u8 u16 u32 u64 usize`<br>
usize acts the same way as C's `size_t` or Rust's `usize` type

### Signed
`i8 i16 i32 i64 int`<br>
int acts the same way as C's `int` type

### Os dependant
`int usize char`<br>
`char` is either `u8` or `i8`, acts the same as C's `char` type

## Floats
`f32 f64`

## Boolean
`bool | true false`

## Pointers
`^ | &`<br>
`&` works exactly like in C<br>
`^` works like `*` in C but the `^` goes before any type. `^char` or `^int` are pointers to their respective type.<br>
To dereference, `<name>^`

## Void Pointer
`^_`, works exactly like C's `void*`

## Typeid
`typeid T`<br>
`typeid` is the keyword<br>
These are used to pass types to functions, used as generics, used in structs (although `typeid` keyword is implicit in structs)

# Types "shipped" with Impulse
`option result string dyn array`

## Options
These work similar to Rust's Options.<br>
`option[int] index;`<br>
Option has two fields, a generic `value` and a boolean `none`

## Result
These work similar to Rust's Results.<br>
`result[int] var;`<br>
Result has two fields, a generic `value` and a cstr (^char) `error`

## String
`string word: str.from("hello world");`<br>
`mem.dealloc(word.data)`<br>
Memory allocated string that contains a cstr (^char) `data`, a usize `len` and a usize `cap`<br>
Note: all memory allocated data will be under a `data` field in the standard library

## Dyn
`dyn[int] nums: dyn.new(int);`<br>
`mem.dealloc(nums.data);`<br>
Memory allocated dynamic array that contains a generic pointer `data`, a usize `len` and a usize `cap`

## Array
`@array[100] int nums: |1 2 3 4 5|;`<br>
`@array int nums: |1 2 3 4 5|;`<br>
Array's can also be represented through pointers but this one has a generic pointer `data` and a usize `len`.
