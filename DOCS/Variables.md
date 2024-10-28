# Variables
These work similarly to C where it's `<type> <name>: <value>`, also note that `:` is the assignment operator and `=` is the equality operator
```
int x: [5 + 5];
int y; # this will always be 0
int z: @garbage; # this will be an uninitalised value

[10]int arr: |0 1 2|;
dyn[int] nums: dyn.new(int);

^int pointer_to_x: &x;
x^: 15;
```

# Constant variable
`::` is the constant assignment operator, similar to Odin or Jai
```
int x :: 10;
x: 15; # errors
```

# Type Inference
The `let` keyword will try to infer the type of the variable as long as there's enough information about the type
```
let x: 10; # the type would be int
let word: str.from("hello"); # the type would be str

let something: |10 15|; # this would error since there isn't enough info on the type
```

# Note
Almost all identifiers can have a `.` in it. This is to emulate methods or fields without a struct if you just need the association.<br>
Can be useful for pointers as they don't have a length,
```
^int nums: mem.alloc(int 10);
usize nums.len: 10;
```
