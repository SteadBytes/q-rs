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
