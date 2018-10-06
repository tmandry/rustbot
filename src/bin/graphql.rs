extern crate failure;
extern crate serde;
extern crate graphql_client;
extern crate reqwest;

extern crate rustbot;

use graphql_client::{GraphQLQuery, Response};
use rustbot::{TeamMembersQuery, team_members_query};
use std::env;

fn main() -> Result<(), failure::Error> {
    let variables = team_members_query::Variables {organization: "rust-lang-nursery".to_owned()};
    let query = TeamMembersQuery::build_query(variables);

    let client = reqwest::Client::new();
    let mut res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(env::var("TOKEN")?)
        .json(&query)
        .send()?;

    let response: Response<team_members_query::ResponseData> = res.json()?;
    //println!("{:#?}", response);

    //println!("{}", response.data.expect("no response").viewer.login);

    //for team in response.data.expect("no response").viewer.teams
    let team_data = response.data.expect("no response")
        .organization.expect("could not read organization data")
        .teams.edges.expect("could not read teams data");
    let teams = team_data
        .iter().flatten()
        .map(|edge| &edge.node).flatten();

    for team in teams {
        println!("=== Team: {} ===", team.name);
        let member_info = &team.members.edges.as_ref().expect("could not read team members");
        let members = member_info.iter().flatten().map(|edge| &edge.node);
        for member in members {
            println!("- {} ({})",
                member.login,
                member.name.as_ref().unwrap_or(&member.id));
        }
    }

    Ok(())
}
