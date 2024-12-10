use anyhow::{Context, Result};
use std::fs;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
    #[error("Serialize json error: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("Error: {0:?}")]
    BigError(Box<BigError>), // 如果对象比较大的时候, 不要在栈上分配. 而是用Box包裹, 在堆上分配
    #[error("Custom error: {0}")]
    Custom(String),
}

#[allow(unused)]
#[derive(Debug)]
struct BigError {
    a: String,
    b: Vec<String>,
    c: [u8; 64],
    d: u64,
}

fn main() -> Result<()> {
    println!("size of MyError is {}", size_of::<MyError>());

    // 这里能这么做的原因是, Io error 能够转换成MyError
    let filename = "nonexistent_file.txt";
    let fd =
        fs::File::open(filename).with_context(|| format!("can not find file: {}", filename))?;
    fail_with_error()?;

    Ok(())
}

fn fail_with_error() -> Result<(), MyError> {
    Err(MyError::Custom("This is a custom error".to_string()))
}
