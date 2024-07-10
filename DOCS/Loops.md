# Loops

Impulse has two types of loops. `loop` which is similar to a while loop, and `for` which is a for in loop

## Loop (While)
This is probably the most common way to write this loop
```
loop (i < 10) [+] {
    # code...
}
```
There's a few things to disect here. `(i < 10)`, the `i` is an implicit variable declaration of type `usize`<br>
This basically means "while i is smaller than 10", anything in the brackets needs to equate to a `bool`<br>
Now what's the `[+]`? It's like a shorthand for `i++`, and that means `[-]` works as `i--`. Note that if you use `[-]`,
you need to declare you're variable beforehand<br><br>

If you don't want to automatically increase or decrease after each loop, you can use `[_]` or just simply don't include the square brackets so it looks like
```
loop (i < 10) {
    # code...
    i: [i + 2];
}
```

## For
Impulse for loops look similar to Zig's ones
Here's an example
```
@array int nums: |1 2 3 4 5|;

for (nums) [num] {
    @c [printf("%d\n", num);];
}
```
Inside the brackets is where you put your array. Implicit variable declarations in the square brackets. `note that for loops work for arrays, dynamic arrays and strings`<br>

If you want the current index, simply declare in the square brackets after your element declaration
```
for (nums) [num i] {
    @c [printf("num: %d, index: %d\n", num, i);];
}
```
<a href="./Generics.md">Next -> Generics</a>
