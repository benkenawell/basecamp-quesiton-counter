use serde::{Serialize, Deserialize};


pub fn collect_answer_data(client: &reqwest::blocking::Client) -> Result<Vec<Answer>, reqwest::Error> {

    // 1. get the user information
    let user_info = client.get("https://launchpad.37signals.com/authorization.json")
        .send()?
        .json::<AuthEndpoint>()?;
    // println!("auth response {:#?}", user_info);

    // 2. get the projects of the user
    let base_url = &user_info.accounts[0].href;
    let project_url = [base_url.to_string(), "/projects.json".to_string()].concat();
    let projects = client.get(&project_url).send()?.text()?;
    // println!("projects {:#?}", projects); // prints excessive content
    let projects_json: serde_json::Value = serde_json::from_str(&projects).unwrap(); // could create the concrete type to parse to instead here
    // println!("Project: {} \n Purpose: {}",projects_json[0]["name"], projects_json[0]["purpose"]);

    // print out the names of all the projects
    // let project_names: Vec<&str> = projects_json.as_array().unwrap().iter()
    //     .map(|project| project["name"].as_str().unwrap()).collect(); // must use as_str, NOT to_string
    // println!("All project names: {:?}", project_names );
    
    // get the project I want
    let family_project = projects_json.as_array().unwrap().iter()
        .find(|project| project["name"].as_str().unwrap().eq(&String::from("Family"))).unwrap();
    // println!("Family Project: {}", family_project);
    
    // print all names from the dock
    // let dock_names: Vec<&str> = family_project["dock"].as_array().unwrap().iter()
    //     .map(|dock_item| dock_item["name"].as_str().unwrap()).collect();
    // println!("dock names {:?}", dock_names);

    // use the dock property to find the "questionnaire" value of the "name" property
    let questionnaire = family_project["dock"].as_array().unwrap().iter()
        .find(|dock| dock["name"].as_str().unwrap().eq(&String::from("questionnaire"))).unwrap();
    // println!("Questionnaire: {}", questionnaire);

    // 3. get the questionnaire so I have the questions url
    let questionnaire_url = questionnaire["url"].as_str().unwrap().to_string();
    let questionnaire_info = client.get(&questionnaire_url).send()?.text()?;
    // println!("\n questionnaire info {}", questionnaire_info);
    let questionnaire_json: serde_json::Value = serde_json::from_str(&questionnaire_info).unwrap();

    // 4. get the questions
    let questions_url = questionnaire_json["questions_url"].as_str().unwrap().to_string();
    let questions = client.get(&questions_url).send()?.text()?;
    let questions_json: serde_json::Value = serde_json::from_str(&questions).unwrap();
    // println!("questions! {:?}", questions);

    // find the running question I'm looking for
    let run_question = questions_json.as_array().unwrap().iter()
        .find(|question| question["title"].as_str().unwrap().eq(&String::from("Did you get to run today?"))).unwrap();
    // println!("question: {:?} \nanswer url: {:?}", run_question["title"], run_question["answers_url"]);

    // 5. get the answers, and parse!
    let answer_url = run_question["answers_url"].as_str().unwrap().to_string();
    let answers_resp: reqwest::blocking::Response = client.get(&answer_url).send()?;
    let mut answers_headers = answers_resp.headers().clone();
    let mut answers_body = answers_resp.json::<Vec<Answer>>()?;
    // println!("answers! {:?}", answers_body);

    // loop through all the paginated answers until we have them all!
    let mut link_header = answers_headers.get("link");
    loop {
        match link_header {
            None => break,
            Some(head) => {
                let new_page = client.get(extract_link_header(head)).send()?;
                answers_headers = new_page.headers().clone();
                let mut new_page_body = new_page.json::<Vec<Answer>>()?;
                link_header = answers_headers.get("link");
                answers_body.append(&mut new_page_body);
            }
        }
    }

    Ok(answers_body)
}

/// manual extraction of the next page link in the Link value of the header
// Maybe I should just ask for the whole HeaderMap and extract the link header myself?
fn extract_link_header(link_header_value: &reqwest::header::HeaderValue) -> &str {
    link_header_value.to_str().unwrap().split_terminator(";").take(1).collect::<Vec<&str>>()
        .first().unwrap().strip_prefix("<").unwrap().strip_suffix(">").unwrap()
}

/// struct used to parse the answers endpoint in step 5 of the API calls
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Answer {
    id: u64,
    status: String,
    visible_to_clients: bool,
    created_at: String,
    updated_at: String,
    title: String,
    inherits_status: bool,
    r#type: String,
    url: String,
    app_url: String,
    bookmark_url: String,
    subscription_url: String,
    comments_count: u64,
    comments_url: String,
    parent: Parent,
    bucket: Bucket,
    creator: Creator,
    content: String,
    group_on: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Parent {
    id: u64,
    title: String,
    r#type: String,
    url: String,
    app_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Bucket {
    id: u64,
    name: String,
    r#type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Creator {
    id: u64,
    attachable_sgid: String,
    name: String,
    email_address: String,
    personable_type: String,
    title: Option<String>,
    bio: Option<String>,
    created_at: String,
    updated_at: String,
    admin: bool,
    owner: bool,
    client: bool,
    time_zone: String,
    avatar_url: String,
    company: Company,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Company {
    id: u64,
    name: String,
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