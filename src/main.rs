use ecosystem::MyError;
use std::fs;
use anyhow::{Context, Result};

fn main() -> Result<()>{
    println!("size of MyError is {}", size_of::<MyError>());

    // 这里能这么做的原因是, Io error 能够转换成MyError
    let filename = "nonexistent_file.txt";
    let fd = fs::File::open(filename)
        .with_context(|| format!("can not find file: {}", filename))?;
    fail_with_error()?;

    Ok(())
}

fn fail_with_error() -> Result<(), MyError>{
    Err(MyError::Custom("This is a custom error".to_string()))
}
