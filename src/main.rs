use std::fs;

use anyhow::Context;
use ecosystem::MyError;

fn main() -> Result<(), anyhow::Error> {
    println!("Hello, world!");
    let filename = "README.md";
    let _fd = fs::File::open(filename).with_context(|| {
        println!("hhhhh");
        "can not find file"
    })?;
    fail_with_error()?;
    Ok(())
}

fn fail_with_error() -> Result<(), MyError> {
    println!("{}", std::mem::size_of::<MyError>());
    println!("{}", std::mem::size_of::<String>());
    Err(MyError::Custom("This is a custom error".to_string()))
}
