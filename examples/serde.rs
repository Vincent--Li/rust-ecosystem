use anyhow::Result;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Builder)]
struct User {
    name: String,
    age: u8,
    skills: Vec<String>,
}

fn main() -> Result<()> {
    let user = UserBuilder::default()
        .name("John".to_string())
        .age(42)
        .skills(vec!["Rust".to_string(), "C++".to_string()])
        .build()?;

    let json = serde_json::to_string(&user)?;
    println!("{}", json);

    let user2 = serde_json::from_str::<User>(&json)?;
    assert_eq!(user, user2);

    Ok(())
}
