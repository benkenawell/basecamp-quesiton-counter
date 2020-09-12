# Project Basecamp

I have been keeping track of the days I've run an when I haven't via a question Basecamp asks me everyday.

The goal of this project is to download the answers to those questions collect them and come up with some insights, while learning Rust along the way!  There are probably better languages to do this in, but if it becomes really apparent, maybe I'll try some FFI.

## OAuth

BaseCamp 3 requires OAuth 2 to access their API.  This is much more difficult than a simple Access Token, but worth the extra security.  However, it seems BaseCamp does not return a valid "token_type" when using the oauth2 crate's BasicClient.  Luckily the Wunderlist example from their site has a SpecialClient I've used here to get around that.  Also worth noting is that BaseCamp expects client id and client secret to be part of the request body when exchanging the code for an access token.  This is not the default option for oauth2's BasicClient.  But with the help of all the example's the oauth2 crate gave me, I was able to get it up and running!

The config for the oauth I have stored in a local json config file.  Obviously this can't go into version control, but it's nice to have it stored close by!  Additionally, it was a good starter on serializing json, which will be a lot of this program once I interact with the Basecamp API.