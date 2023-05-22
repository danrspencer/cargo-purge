mod private_module;
pub mod public_module;

pub fn public_hello_1() {
    println!("Hello, world!");
}

pub fn public_hello_2() {
    println!("Hello, world!");
}

pub fn public_hello_3() {
    println!("Hello, world!");
}

fn private_hello() {
    println!("Hello, Dave!");
}
