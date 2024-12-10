use thiserror::Error;

#[derive(Error, Debug)]
enum MyError {
    #[error("Error")]
    Error,
}