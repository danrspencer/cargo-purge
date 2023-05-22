mod private;
pub mod public;

fn main() {
    // Inline import via fully qualified path
    package_1::public_hello_3();
}

#[cfg(test)]
mod test {
    use super::*;
    use package_1::public_module::public_hello;
}
