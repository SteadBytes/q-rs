# q

`my-project/src/main.rs`:

```rust
#[macro_use]
extern crate q;

fn hello(name: &str) -> String {
  q!(name);
  format!("Hello, {}!", name)
}

fn main() {
  q!()

  let name = "SteadBytes";

  q!(name);

  let greeting = q!(hello(name));

  q!(Some(42));
}
```

Produces the following in `/tmp/q`:

```
[07:32:45 src/main.rs:8 my_project::main::main]
0.000s >
0.000s > name = "SteadBytes"
[07:32:45 src/main.rs:5 my_project::main::hello]
0.0001s > name = "SteadBytes"
[07:32:45 src/main.rs:14 my_project::main::main]
0.0001s > hello(name) = "Hello, SteadBytes!"
0.0001s > Some(42)
```

A header line is logged at a regular (configurable) interval *or* if the calling
function or module has changed.

Expression values are returned.
