# Functions
I called this file functions but in impulse, we don't care what they're called. whether it'd be functions, procedures, methods, whatever.
In fact, there's no name for them in impulse. just ().
<br>
here's your basic Hello, World! in impulse.
```
_() main: {
    print("Hello, World!")
}
```
This may look a bit alien for new programmers but it's nothing hard to understand.
<br>
the `_` is the type, which means void. `main` is the name. and the `:` is for assign to the code block (more spiritually than anything honestly)
<br>
if you are following the documentation file by file, you'd have read about this "assigment philosophy" in the variables doc. <a href="./Variables.md">if not</a>

## IMPORTANT NOTE
The spirit of this language is impulse and bogding. You wouldn't think there's nothing else special about these?
```
main(): _ {}
_(): main {}
(): main _ {}
:() _ main {}
_() main: {}
```
These are all valid in impulse. the compiler doesn't care what order you put them in, as long as all the stuff you need to make a function is there. That does mean tho that no other syntax in this language can be written similarly to this or the compiler might make an oopsie.
<br>
Oh and since everyone loves `idiomatic` ways to do things in languages, this is the idiomatic way to make functions in impulse (all of the docs will show the idiomatic syntax of impulse)
```
_() main: {}
```
