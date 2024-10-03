# Quick Start
## Installation
-- Note: Tested on Windows and MacOS so tread carefully on other OS's --

### Requirements
GNU Compiler Collection (gcc) - <a href="https://gcc.gnu.org/install/binaries.html">https://gcc.gnu.org/install/binaries.html</a><br>
Git - <a href="https://git-scm.com/downloads">https://git-scm.com/downloads</a>

### Download Steps
Open a terminal:<br>
`$ cd ~`<br>
`$ git clone https://github.com/AyeAreEm/impulse.git`<br>
`$ cd impulse`<br>

Add where this project is located in the `.cargo/config.toml` file<br>
There will be a `current_path` variable, set it to `your/path/to/impulse` (should end with /impulse)<br>

`$ cargo build --release`<br>
Make sure to add the executable to your path<br>

Now run `impulse` to check if it has been correctly added to your path
`$ impulse`

## Your first Impulse file
Create `hello.imp` and inside:
```
_ main() :: {
    println("hello world");
}
```
To generate .exe, `impulse -b hello.imp hello`<br>
To generate .c and .exe `impulse -b -c hello.imp hello`<br> (this will generate `output.c`)
To generate just .c, `impulse -c hello.imp hello` (this will generate `hello.c`)

For more information, check the <a href="./Docs.md">Docs</a>, <a href="./Overview.md">Overview</a> and or <a href="../examples">Examples</a>. Happy Hacking
