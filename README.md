# q

Quick and dirty debugging for tired Rust programmers.

## Example Usage

[`examples/simple.rs`](examples/simple.rs)

```rust
#[macro_use]
extern crate q;

fn hello(name: &str) -> String {
    q!(name);
    format!("Hello, {}!", name)
}

fn main() {
    // No message
    q!();

    // Identifier
    let name = "SteadBytes";
    q!(name);

    // Returns expression values
    let greeting = q!(hello(name));
    q!(greeting);

    q!(Some(42));
}
```

Running the above using `cargo run --example simple` writes the following to
`$TMP_DIR/q` (`/tmp/q` on Linux):

```
[19:13:49 ThreadId(1) examples/simple.rs simple::main:11]
>
> name = "SteadBytes"
[19:13:49 ThreadId(1) examples/simple.rs simple::hello:5]
> name = "SteadBytes"
[19:13:49 ThreadId(1) examples/simple.rs simple::main:18]
> hello(name) = "Hello, SteadBytes!"
> greeting = "Hello, SteadBytes!"
> Some(42) = Some(42)
```

A header line is logged at a regular (configurable) interval _or_ if the calling
function or module has changed.

Expression values are returned by `q` invocations.

## Inspired by

- [`q` for Python](https://github.com/zestyping/q)
- [`q` for Golang](https://github.com/ryboe/q)

## Why is this crate called `q-debug` and not `q`?

`q` is unfortunately [taken already](https://crates.io/crates/q) (though I'm not
sure what it is...).


