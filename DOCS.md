# Documentation
-- WARNING: UNDER DEVELOPMENT, MAY CHANGE --

## Types
Impulse has the basics.
<br>
int -> 32 bit integer
string -> dynamic array of chars / 8 bit unsigned integers
_ -> void

## Functions / Procedures
We don't care what they're called, even to the point where it isn't included in the language
To create a function that returns void or nothing. it'd look something like this:
```
_(): main {
    print("hello world")
}
```
The underscore means that this function returns void.
The brackets / parentheses are just a way of holding parameters for this function.
You might notice there's a colon, well that's the assigment operator in Impulse. that's right, we don't use `=` but instead `:` 
This means making a variable looks like

```
int age: 18
string name: "Lorem"
bool is_male: true
```

## If statement
Two things that are notable here. Since the `:` is used for assigment, we use `=` for equality.
Also, instead of `else if` or `elif`, we use `or`. Why? idk fight me.
```
int x: 10

if x = 10 {
    do_smth()
} or x = 20 {
    do_smth_else()
} else {
    print("womp")
}
```

## Loop statement 
-- Still Thinking This One Over --
There's always been the `for` and `while` loop which accomplish practically the same thing.
So ours is like so:
```
_(): main {
    loop int x: 0 < 10 +{
        print("hello")
    }

    loop bool running: true {
        print("ahhhhhhhh")
    }
}
```
