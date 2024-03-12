# Variables
I don't like keywords like `let` or `var`, that's why impulse's approach to variables is similar to C, Odin and Jai. Here's some examples:
```
int x: 10
string msg: "Hello"
bool ahh: true # couldn't think of a name

[]int arr: [0 1 2]
```
There's a few things to note here. First, the `:` - this is the assigment operator in impulse. And yes, that does mean that the equality operator is `=` and not `==`
<br>
Second, no comma's. the compiler understand what you're trying to do, but that does mean you need to be more specific when saying something is one element by wrapping it around `(0 + 1)`. This is `SUBJECT TO CHANGE`
Next, which is a big part of impulse's design philosophy, the order of the code.
```
<TYPE> <NAME> <VALUE>
```
This only applies when declaring something, anything in impulse, even functions. There's a catch though, getting / using / re-assigning / pretty much anything besides declaring is written differently. It's unnoticable in most code
```
x: 20
msg: "World"
ahh: false

arr[0]: 1
arr[1]: 2
arr[2]: 3
```

<a href="./Functions.md">Next -> Functions</a>
