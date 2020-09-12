use std::collections::HashMap;

mod oauth;

// return type is a Result type with a Unit Ok, or any type that implements the Error trait object methods (boxed because trait object)
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    let at = oauth::get_auth_token();
    println!("main at {:?}", at);
    println!("{:#?}", get_data()?);
    Ok(())
}

fn get_data() -> Result<HashMap<String, String>, reqwest::Error> {
    reqwest::blocking::get("https://httpbin.org/ip")?
    .json::<HashMap<String, String>>()
}