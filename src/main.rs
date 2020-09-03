use std::collections::HashMap;

// return type is a Result type with a Unit Ok, or any type that implements the Error trait object methods (boxed because trait object)
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    // alternative, all in one line
    // Ok(print_json(get_data()?))

    // another alternative, all in one line, I like this one better than above
    // Ok(println!("{:#?}", get_data()?))

    println!("{:#?}", get_data()?)
    Ok(())
}

fn get_data() -> Result<HashMap<String, String>, reqwest::Error> {
    reqwest::blocking::get("https://httpbin.org/ip")?
    .json::<HashMap<String, String>>()
}

fn print_json(hash: HashMap<String, String>) -> () {
    println!("{:#?}", hash);
}