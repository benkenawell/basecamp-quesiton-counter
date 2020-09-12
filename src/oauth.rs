// A amaglamation of oauth2 library's Github, Wunderlist, and Microsoft's examples

use oauth2::basic::{BasicErrorResponse, BasicTokenType};
use oauth2::helpers;
use oauth2::TokenType;
use std::time::Duration;
use oauth2::reqwest::http_client;
use oauth2::{
    AuthUrl, AuthType, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl, PkceCodeChallenge, AccessToken, Client, 
    EmptyExtraTokenFields, ExtraTokenFields, RefreshToken, 
};
// use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use url::Url;
use serde::{Deserialize, Serialize};



type SpecialTokenResponse = NonStandardTokenResponse<EmptyExtraTokenFields>;
type SpecialClient = Client<BasicErrorResponse, SpecialTokenResponse, BasicTokenType>;

fn default_token_type() -> Option<BasicTokenType> {
    Some(BasicTokenType::Bearer)
}

///
/// Non Standard OAuth2 token response.
///
/// This struct includes the fields defined in
/// [Section 5.1 of RFC 6749](https://tools.ietf.org/html/rfc6749#section-5.1), as well as
/// extensions defined by the `EF` type parameter.
/// In this particular example token_type is optional to showcase how to deal with a non
/// compliant provider.
///
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NonStandardTokenResponse<EF: ExtraTokenFields> {
    access_token: AccessToken,
    // In this example wunderlist does not follow the RFC specs and don't return the
    // token_type. `NonStandardTokenResponse` makes the `token_type` optional.
    #[serde(default = "default_token_type")]
    token_type: Option<BasicTokenType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<RefreshToken>,
    #[serde(rename = "scope")]
    #[serde(deserialize_with = "helpers::deserialize_space_delimited_vec")]
    #[serde(serialize_with = "helpers::serialize_space_delimited_vec")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    scopes: Option<Vec<Scope>>,

    #[serde(bound = "EF: ExtraTokenFields")]
    #[serde(flatten)]
    extra_fields: EF,
}

impl<EF> TokenResponse<BasicTokenType> for NonStandardTokenResponse<EF>
where
    EF: ExtraTokenFields,
    BasicTokenType: TokenType,
{
    ///
    /// REQUIRED. The access token issued by the authorization server.
    ///
    fn access_token(&self) -> &AccessToken {
        &self.access_token
    }
    ///
    /// REQUIRED. The type of the token issued as described in
    /// [Section 7.1](https://tools.ietf.org/html/rfc6749#section-7.1).
    /// Value is case insensitive and deserialized to the generic `TokenType` parameter.
    /// But in this particular case as the service is non compliant, it has a default value
    ///
    fn token_type(&self) -> &BasicTokenType {
        match &self.token_type {
            Some(t) => t,
            None => &BasicTokenType::Bearer,
        }
    }
    ///
    /// RECOMMENDED. The lifetime in seconds of the access token. For example, the value 3600
    /// denotes that the access token will expire in one hour from the time the response was
    /// generated. If omitted, the authorization server SHOULD provide the expiration time via
    /// other means or document the default value.
    ///
    fn expires_in(&self) -> Option<Duration> {
        self.expires_in.map(Duration::from_secs)
    }
    ///
    /// OPTIONAL. The refresh token, which can be used to obtain new access tokens using the same
    /// authorization grant as described in
    /// [Section 6](https://tools.ietf.org/html/rfc6749#section-6).
    ///
    fn refresh_token(&self) -> Option<&RefreshToken> {
        self.refresh_token.as_ref()
    }
    ///
    /// OPTIONAL, if identical to the scope requested by the client; otherwise, REQUIRED. The
    /// scipe of the access token as described by
    /// [Section 3.3](https://tools.ietf.org/html/rfc6749#section-3.3). If included in the response,
    /// this space-delimited field is parsed into a `Vec` of individual scopes. If omitted from
    /// the response, this field is `None`.
    ///
    fn scopes(&self) -> Option<&Vec<Scope>> {
        self.scopes.as_ref()
    }
}

pub fn get_auth_token() -> String { // TODO, should probably return the Access Token, not a String
  // DO NOT ADD TO VCS
  let client_id = ClientId::new("".to_string());
  let client_secret = ClientSecret::new("".to_string());
  let auth_url = AuthUrl::new("https://launchpad.37signals.com/authorization/new?type=web_server".to_string())
    .expect("Invalid authorization endpoint URL");
  let token_url = TokenUrl::new("https://launchpad.37signals.com/authorization/token?type=web_server".to_string())
    .expect("Invalid token endpoint URL");

  // Set up the config for the Github OAuth2 process.
  // let client = BasicClient::new(
  let client = SpecialClient::new(
    client_id,
    Some(client_secret),
    auth_url,
    Some(token_url),
  )
  .set_auth_type(AuthType::RequestBody)
  // This example will be running its own server at localhost:8080.
  // See below for the server implementation.
  .set_redirect_url(
    RedirectUrl::new("http://localhost:8080".to_string()).expect("Invalid redirect URL"),
  );

  // Generate a PKCE challenge.
  let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

  // Generate the authorization URL to which we'll redirect the user.
  let (authorize_url, csrf_state) = client
    .authorize_url(CsrfToken::new_random)
    // This example is requesting access to the user's public repos and email.
    // .add_scope(Scope::new("read".to_string()))
    .set_pkce_challenge(pkce_challenge)
    .url();

  println!(
      "Open this URL in your browser:\n{}\n",
      authorize_url.to_string()
  );

  // A very naive implementation of the redirect server.
  let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
  for stream in listener.incoming() {
    if let Ok(mut stream) = stream {
      let code;
      let state;
      {
        let mut reader = BufReader::new(&stream);

        let mut request_line = String::new();
        reader.read_line(&mut request_line).unwrap();

        let redirect_url = request_line.split_whitespace().nth(1).unwrap();
        let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

        let code_pair = url
          .query_pairs()
          .find(|pair| {
              let &(ref key, _) = pair;
              key == "code"
          })
          .unwrap();

        let (_, value) = code_pair;
        code = AuthorizationCode::new(value.into_owned());

        let state_pair = url
          .query_pairs()
          .find(|pair| {
              let &(ref key, _) = pair;
              key == "state"
          })
          .unwrap();

        let (_, value) = state_pair;
        state = CsrfToken::new(value.into_owned());
      }

      let message = "Go back to your terminal :)";
      let response = format!(
        "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
        message.len(),
        message
      );
      stream.write_all(response.as_bytes()).unwrap();

      println!("Basecamp returned this structure: \n{:#?}\n", code);
      println!("Basecamp returned the following code:\n{}\n", code.secret());
      println!(
        "Basecamp returned the following state:\n{} (expected `{}`)\n",
        state.secret(),
        csrf_state.secret()
      );

      // Exchange the code with a token.
      let token_res = client.exchange_code(code).set_pkce_verifier(pkce_verifier).request(http_client);
      println!("client {:?}", client);
      println!("token_res {:?}", token_res);

      println!("Basecamp returned the following token:\n{:?}\n", token_res);

      if let Ok(token) = token_res {
          println!("access token {:?}", token.access_token().secret());
          // NB: Github returns a single comma-separated "scope" parameter instead of multiple
          // space-separated scopes. Github-specific clients can parse this scope into
          // multiple scopes by splitting at the commas. Note that it's not safe for the
          // library to do this by default because RFC 6749 allows scopes to contain commas.
          let scopes = if let Some(scopes_vec) = token.scopes() {
              scopes_vec
                  .iter()
                  .map(|comma_separated| comma_separated.split(','))
                  .flatten()
                  .collect::<Vec<_>>()
          } else {
              Vec::new()
          };
          println!("Basecamp returned the following scopes:\n{:?}\n", scopes);

          return token.access_token().secret().to_string();
      }

      // The server will terminate itself after collecting the first code.
      break;
      
    }
  }
  String::from("")
}