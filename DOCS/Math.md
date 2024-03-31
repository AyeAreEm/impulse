# Math

There is one major thing to mention and I wasn't sure where to write it in the docs.
```
# this won't work if you want to assign an int that needs arithmetic operations
int x: 10 + 10

# this is how to do it
int x: [10+10]

# you can also nest them
int x: [[1 + 3] * [3 * 3]]

# of course this works as well
int y: 10
int x: [x + 5]
```
