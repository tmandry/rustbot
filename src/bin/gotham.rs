extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate oauth2;
extern crate mime;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate url;

use std::env;
use gotham::http::response::create_response;
use gotham::state::State;
use hyper::{Response, StatusCode};

use gotham::router::Router;
use gotham::router::builder::*;

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct OAuthResponse {
    code: String,
    state: String,
}

fn router() -> Router {
    build_simple_router(|route| {
        route.get("/")
             .with_query_string_extractor::<OAuthResponse>()
             .to(say_hello);
    })
}

/// Create a `Handler` which is invoked when responding to a `Request`.
///
/// How does a function become a `Handler`?.
/// We've simply implemented the `Handler` trait, for functions that match the signature used here,
/// within Gotham itself.
pub fn say_hello(state: State) -> (State, Response) {
    let message = {
        let oauth_info = state.borrow::<OAuthResponse>();

        println!("Github returned the following code:\n{}\n", oauth_info.code);
        println!("Github returned the following state:\n{}\n", oauth_info.state);

        // Exchange the code with a token.
        let token = oauth_config().exchange_code(oauth_info.code.clone());

        println!("Github returned the following token:\n{:?}\n", token);

        format!("Hello World! {} {}\n{:?}", oauth_info.code, oauth_info.state, token)
    };
    let res = create_response(
        &state,
        StatusCode::Ok,
        Some((message.into_bytes(), mime::TEXT_PLAIN)),
    );

    (state, res)
}

/// Start a server and call the `Handler` we've defined above for each `Request` we receive.
pub fn main() {
    let addr = "127.0.0.1:8080";
    println!("Point your browser to {}", oauth_config().authorize_url().to_string());
    println!("Listening for requests at http://{}", addr);
    gotham::start(addr, router())
}

fn oauth_config() -> oauth2::Config {
    let github_client_id = env::var("GITHUB_CLIENT_ID").expect("Missing the GITHUB_CLIENT_ID environment variable.");
    let github_client_secret = env::var("GITHUB_CLIENT_SECRET").expect("Missing the GITHUB_CLIENT_SECRET environment variable.");
    let auth_url = "https://github.com/login/oauth/authorize";
    let token_url = "https://github.com/login/oauth/access_token";

    // Set up the config for the Github OAuth2 process.
    let mut config = oauth2::Config::new(github_client_id, github_client_secret, auth_url, token_url);

    // This example is requesting access to the user's public repos and email.
    config = config.add_scope("public_repo");
    config = config.add_scope("user:email");

    // This example will be running its own server at localhost:8080.
    // See below for the server implementation.
    config = config.set_redirect_url("http://localhost:8080");

    // Set the state parameter (optional)
    config = config.set_state("1234");

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use gotham::test::TestServer;

    #[test]
    fn receive_hello_world_response() {
        let test_server = TestServer::new(|| Ok(say_hello)).unwrap();
        let response = test_server
            .client()
            .get("http://localhost")
            .perform()
            .unwrap();

        assert_eq!(response.status(), StatusCode::Ok);

        let body = response.read_body().unwrap();
        assert_eq!(&body[..], b"Hello World!");
    }
}
