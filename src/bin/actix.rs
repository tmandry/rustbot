extern crate actix_web;
extern crate env_logger;
extern crate oauth2;
#[macro_use]
extern crate serde_derive;
use actix_web::{http, middleware, server, App, Responder, Query};
use std::env;

#[derive(Deserialize)]
struct OAuthResponse {
    code: String,
    state: String,
}

fn index(oauth_info: Query<OAuthResponse>) -> impl Responder {
    println!("Github returned the following code:\n{}\n", oauth_info.code);
    println!("Github returned the following state:\n{}\n", oauth_info.state);

    // Exchange the code with a token.
    let token = oauth_config().exchange_code(oauth_info.code.clone());

    println!("Github returned the following token:\n{:?}\n", token);

    format!("Hello World! {} {}\n{:?}", oauth_info.code, oauth_info.state, token)
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

fn main() {
    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    println!("Point your browser to {}", oauth_config().authorize_url().to_string());

    server::new(|| {
        App::new()
            .middleware(middleware::Logger::default())
            .route("/", http::Method::GET, index)
    }).bind("localhost:8080")
      .unwrap()
      .run();
}
