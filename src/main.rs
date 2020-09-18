use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use serde::{Serialize, Deserialize};
use reqwest::header::{AUTHORIZATION, HeaderValue, };
use oauth2::{ TokenResponse, };

mod oauth;
mod api;

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
    // println!("json {:?}", json);

    let at = oauth::get_auth_token(json.client_id, json.client_secret)?;
    // println!("main at {:?}", at);

    let client = build_client(at.access_token())?;

    let answers_body = api::collect_answer_data(&client)?;

    println!("length of answers {}", answers_body.len());
    // now I have all of the data... now what?


    Ok(())
}

fn build_client (at: &oauth2::AccessToken) -> Result<reqwest::blocking::Client, reqwest::Error> {
    let mut bearer_token = "Bearer ".to_string();
    bearer_token.push_str(&at.secret().to_string());
    let mut auth_header = reqwest::header::HeaderMap::new();
    auth_header.insert(AUTHORIZATION, HeaderValue::from_str(&bearer_token).unwrap());
    reqwest::blocking::ClientBuilder::new().default_headers(auth_header).user_agent("Run Counter").build()
}

// fn get_auth_endpoint(at: &oauth2::AccessToken) -> Result<AuthEndpoint, reqwest::Error>{
//     // now I have two methods here that do the same thing, but in different ways

//     // method one: now I have a client with the token already set up, could be reused
//     let mut bearer_token = "Bearer ".to_string();
//     bearer_token.push_str(&at.secret().to_string());
//     let mut auth_header = reqwest::header::HeaderMap::new();
//     auth_header.insert(AUTHORIZATION, HeaderValue::from_str(&bearer_token).unwrap());
//     let client: reqwest::blocking::Client = reqwest::blocking::ClientBuilder::new().default_headers(auth_header).build()?;
//     let resp = client.get("https://launchpad.37signals.com/authorization.json").send()?;

//     // method two: very straightforward, easy to make a single request
//     // let resp: reqwest::blocking::Response = reqwest::blocking::Client::new()
//     //     .get("https://launchpad.37signals.com/authorization.json")
//     //     .bearer_auth(&at.secret().to_string())
//     //     .send()?;
//     // println!("auth response {:#?}", resp.text());
//     // println!("auth response {:#?}", resp.json::<AuthEndpoint>());

//     resp.json::<AuthEndpoint>()
// }