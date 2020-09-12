use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::collections::HashMap;
use serde_json::{Value, from_str};

mod oauth;

// return type is a Result type with a Unit Ok, or any type that implements the Error trait object methods (boxed because trait object)
fn main() -> Result<(), Box<dyn std::error::Error>> {

    // read json file
    let file = File::open("./src/basecamp.json")?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    let json: Value = from_str(&contents).expect("JSON was not well-formatted");
    println!("json {:?}", json);

    println!("Hello, world!");
    let at = oauth::get_auth_token(String::from(json["clientId"].as_str().unwrap()), String::from(json["clientSecret"].as_str().unwrap()));
    println!("main at {:?}", at);
    println!("{:#?}", get_data()?);


    Ok(())
}

fn get_data() -> Result<HashMap<String, String>, reqwest::Error> {
    reqwest::blocking::get("https://httpbin.org/ip")?
    .json::<HashMap<String, String>>()
}