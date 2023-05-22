mod private_module;
pub mod public_module;

pub fn public_hello() {
    println!("Hello, world!");
}

fn private_hello() {
    println!("Hello, Dave!");
}
