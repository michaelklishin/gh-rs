extern crate gh;

use std::env;

const ORG: &str = "api-playgrounds";

fn github_token() -> String {
    match env::var("GITHUB_TOKEN") {
        Ok(val)                           => val,
        Err(env::VarError::NotPresent)    => panic!("Please set GITHUB_TOKEN"),
        Err(env::VarError::NotUnicode(_)) => panic!("Please set GITHUB_TOKEN to a valid token")
    }
}

#[test]
fn test_current_user() {
    let t = github_token();
    let c = gh::client(&t);

    let u = c.current_user().
        expect("gh::client::current_user() returned an error");
    assert!(u.id > 1);
}

#[test]
fn test_repos_of_org() {
    let t = github_token();
    let c = gh::client(&t);

    let xs = c.list_repos_of_org(ORG).
        expect("gh::client::list_repos_of_org() returned an error");
    assert!(xs.len() >= 2);
}
