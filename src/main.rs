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
    // println!("json {:?}", json);

    let at = oauth::get_auth_token(json.client_id, json.client_secret)?;
    // println!("main at {:?}", at);

    let client = build_client(at.access_token())?;

    // 1. get the user information
    let user_info = client.get("https://launchpad.37signals.com/authorization.json")
        .send()?
        .json::<AuthEndpoint>()?;
    println!("auth response {:#?}", user_info);

    // 2. get the projects of the user
    let base_url = &user_info.accounts[0].href;
    let project_url = [base_url.to_string(), "/projects.json".to_string()].concat();
    let projects = client.get(&project_url).send()?.text()?;
    // println!("projects {:#?}", projects); // prints excessive content
    let projects_json: serde_json::Value = serde_json::from_str(&projects)?; // could create the concrete type to parse to instead here
    println!("Project: {} \n Purpose: {}",projects_json[0]["name"], projects_json[0]["purpose"]);

    // print out the names of all the projects
    let project_names: Vec<&str> = projects_json.as_array().unwrap().iter()
        .map(|project| project["name"].as_str().unwrap()).collect(); // must use as_str, NOT to_string
    println!("All project names: {:?}", project_names );
    
    // get the project I want
    let family_project = projects_json.as_array().unwrap().iter()
        .find(|project| project["name"].as_str().unwrap().eq(&String::from("Family"))).unwrap();
    println!("Family Project: {}", family_project);
    
    // use the dock property to find the "questionnaire" value of the "name" property
    let questionnaire = family_project["dock"].as_array().unwrap().iter()
        .find(|dock| dock["name"].as_str().unwrap().eq(&String::from("questionnaire"))).unwrap();
    println!("Questionnaire: {}", questionnaire);

    // 3. may not need to get the questionnaires... there might be one per project?

    // 4. get the questions
    let base_url = questionnaire["app_url"].as_str().unwrap();
    let questions_url = [base_url.to_string(), "/questions.json".to_string()].concat();
    let questions = client.get(&questions_url).send()?.text()?; // TODO, why does this return nothing?
    println!("questions! {:?}", questions);

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

fn build_client (at: &oauth2::AccessToken) -> Result<reqwest::blocking::Client, reqwest::Error> {
    let mut bearer_token = "Bearer ".to_string();
    bearer_token.push_str(&at.secret().to_string());
    let mut auth_header = reqwest::header::HeaderMap::new();
    auth_header.insert(AUTHORIZATION, HeaderValue::from_str(&bearer_token).unwrap());
    reqwest::blocking::ClientBuilder::new().default_headers(auth_header).user_agent("Run Counter").build()
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