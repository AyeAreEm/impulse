# Quick Start
## Installation
-- Note: Tested on Windows so tread carefully on other OS's --

### Requirements
GNU Compiler Collection (gcc) - <a href="https://gcc.gnu.org/install/binaries.html">https://gcc.gnu.org/install/binaries.html</a>
Git - <a href="https://git-scm.com/downloads">https://git-scm.com/downloads</a>

### Download Steps
Open a terminal:
`$ cd ~`<br>
`$ git clone https://github.com/AyeAreEm/impulse.git`<br>
`$ cd impulse`<br>
`$ impulse`<br>
Make sure to add the executable to path

## Your first Impulse file
Create `hello.imp`
```
# inside hello.imp

_ main() :: {
    @c [printf("hello world");];
}
```
To generate .exe, `impulse -b hello.imp hello`<br>
To generate .c and .exe `impulse -b -c hello.imp hello`<br> (this will generate `output.c`)
To generate just .c, `impulse -c hello.imp hello` (this will generate `hello.c`)

Congrats, your first hello world in impulse... sorta, I know right now you need to use C Embedding but eventually there will be fmt support in the standard library<br>

For more information, check the <a href="./Docs.md">Docs</a>, <a href="./Overview.md">Overview</a> and or <a href="../examples">Examples</a>. Happy Hacking

