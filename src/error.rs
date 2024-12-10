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
    d: u64
}