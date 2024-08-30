# Loops

## Loop
```
loop (i < 10) [+] {
    # i is type usize here
    # i is out of scope after the closing bracket
}

int i: 0;
loop (i < 10) [+] {

}

int j: 10;
loop (j > 0) [-] {

}

loop (i < 10) {
    i: [i + 2];
}
```

The syntax is `loop (<boolean condition>) <[modifier]> {}`<br>
The modifier is optional. There are 3 modifiers: `[+]`, `[-]`, `[_]`. The plus and minus increment or decrement by one, the void `[_]` is the same as not giving a modifier


## For
Known as the For In or the For Each loop, similar to Zig's syntax
```
@array str fruits: |str.from("apple") str.from("cherry") str.from("banana")|;

for (fruits) [fruit] {
    println("%s" fruit.data);
}

for (fruits) [fruit i] {
    # i is the current index
}
```

The syntax is `for (<elements>) [<elem> <idx>] {}`, the index is optional. Also note that there are only a few types that the for loop supports: `array`, `dyn`, `string`
