# Operators
Both logical operators and bitwise operators share the same syntax<br>
The operators are
```
and or xor ! lshift rshift
```

The way to distinguish if `and` is used as a logical operator instead of a bitwise operator is by the brackets around it<br>
For example, `(10 and 15)` would be `true` or `1` but `[10 and 15]` would be `10`<br>
Some operators however don't work in logical conditions such as `lshift`, `rshift` and `xor`
