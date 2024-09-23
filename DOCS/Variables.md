# Variables
These work similarly to C where it's `<type> <name>: <value>`, also note that `:` is the assignment operator and `=` is the equality operator
```
int x: [5 + 5];
string word: str.from("hello");
bool foo: true;
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
Note: a little quirk with constants are the same as how they are in C, if you get a pointer to it and then change it's value, it will work, no errors.


# Note
Almost all identifiers can have a `.` in it. This is to emulate methods or fields without a struct if you just need the association.<br>
Can be useful for pointers as they don't have a length,
```
^int nums: mem.alloc(int 10);
usize nums.len: 10;
```
