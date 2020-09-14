use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use serde::{Serialize, Deserialize};
use reqwest::header::{AUTHORIZATION, HeaderValue, };
use oauth2::{ TokenResponse, };

mod oauth;

#[derive(Serialize, Deserialize, Debug)]
struct Creds {
    client_id: String,
    client_secret: String,
}

// return type is a Result type with a Unit Ok, or any type that implements the Error trait object methods (boxed because trait object)
fn main() -> Result<(), Box<dyn std::error::Error>> {

    // read json file
    let file = File::open("./src/basecamp.json")?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    let json: Creds = serde_json::from_str(&contents).expect("JSON was not well-formatted");
    println!("json {:?}", json);

    let at = oauth::get_auth_token(json.client_id, json.client_secret)?;
    println!("main at {:?}", at);
    println!("{:#?}", get_data()?);

    let user_info = get_auth_endpoint(at.access_token())?;
    println!("auth response {:#?}", user_info);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct AuthEndpoint {
    expires_at: String,
    identity: Identity,
    accounts: Vec<Account>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Identity {
    id: u64,
    first_name: String,
    last_name: String,
    email_address: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Account {
    product: String,
    id: u64,
    name: String,
    href: String,
    app_href: String,
}

fn get_auth_endpoint(at: &oauth2::AccessToken) -> Result<AuthEndpoint, reqwest::Error>{
    // now I have two methods here that do the same thing, but in different ways

    // method one: now I have a client with the token already set up, could be reused
    let mut bearer_token = "Bearer ".to_string();
    bearer_token.push_str(&at.secret().to_string());
    let mut auth_header = reqwest::header::HeaderMap::new();
    auth_header.insert(AUTHORIZATION, HeaderValue::from_str(&bearer_token).unwrap());
    let client: reqwest::blocking::Client = reqwest::blocking::ClientBuilder::new().default_headers(auth_header).build()?;
    let resp = client.get("https://launchpad.37signals.com/authorization.json").send()?;

    // method two: very straightforward, easy to make a single request
    // let resp: reqwest::blocking::Response = reqwest::blocking::Client::new()
    //     .get("https://launchpad.37signals.com/authorization.json")
    //     .bearer_auth(&at.secret().to_string())
    //     .send()?;
    // println!("auth response {:#?}", resp.text());
    // println!("auth response {:#?}", resp.json::<AuthEndpoint>());

    resp.json::<AuthEndpoint>()
}

#[derive(Serialize, Deserialize, Debug)]
struct Ip {
    origin: String,
}

fn get_data() -> Result<Ip, reqwest::Error> {
    reqwest::blocking::get("https://httpbin.org/ip")?
    .json::<Ip>()
}