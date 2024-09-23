# Primitives
## Void
`_` is void but it's also used as a general symbol to mean "Nothing".

## Integers
### Unsigned
`u8 u16 u32 u64 usize uint`<br>
usize acts the same way as C's `size_t` or Rust's `usize` type<br>
uint acts the same way C's `unsigned int` acts. This means it can be interchangeable in some cases. If you are not sure, assume usize is bigger.

### Signed
`i8 i16 i32 i64 int`<br>
int acts the same way as C's `int` type

### OS dependant
`uint int usize char`<br>
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
`option result str string dyn`

## Options
These work similar to Rust's Options.<br>
`option[int] index;`<br>
Option has two fields, a generic `value` and a boolean `none`

## Result
These work similar to Rust's Results.<br>
`result[int] var;`<br>
Result has two fields, a generic `value` and a cstr (^char) `error`

## Str
`str word: str.from("hello world");`<br>
These are strings that are null terminated that carry the data and the length. `word.len` is to get the length of the string<br>
Note: unlike the `string` type, these are not memory allocated

## String
`string word: string.from("hello world");`<br>
`mem.dealloc(word.data)`<br>
Memory allocated string that are null terminated. It contains a cstr (^char) `data`, a usize `len` and a usize `cap`<br>
Note: all memory allocated data will be under a `data` field in the standard library

## Dyn
`dyn[int] nums: dyn.new(int);`<br>
`mem.dealloc(nums.data);`<br>
Memory allocated dynamic array that contains a generic pointer `data`, a usize `len` and a usize `cap`

## Array
`[100] int nums: |1 2 3 4 5|;`<br>
`[]int nums: |1 2 3 4 5|;`<br>
Array's can also be represented through pointers but this one has a generic pointer `data` and a usize `len`.
