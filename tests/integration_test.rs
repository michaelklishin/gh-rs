extern crate gh;
extern crate rand;

use std::env;
use rand::Rng;
use gh::milestones;

const ORG: &str = "api-playgrounds";
const REPO1: &str = "gh-api-playground";

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

    let u = c.current_user()
        .expect("gh::client::current_user() returned an error");
    assert!(u.id > 1);
}

#[test]
fn test_repos_of_org() {
    let t = github_token();
    let c = gh::client(&t);

    let xs = c.list_repos_of_org(ORG)
        .expect("gh::client::list_repos_of_org() returned an error");
    assert!(xs.len() >= 2);
}

#[test]
fn test_create_and_list_milestones() {
    let c = gh::client(&github_token());

    let xs0 = c.list_milestones(ORG, REPO1)
        .expect("gh::client::list_milestones() returned an error");
    assert!(xs0.len() == 0);

    let t = random_title();
    let m = milestones::MilestoneProperties {
        title: t.clone(),
        state: Some(milestones::State::Open),
        description: None,
        due_on: None
    };
    c.create_milestone(ORG, REPO1, &m)
        .expect("gh::client::create_milestone() returned an error");

    let xs1 = c.list_milestones(ORG, REPO1)
        .expect("gh::client::list_milestones() returned an error");
    assert!(xs1.len() >= 1);

    c.delete_milestone_with_title(ORG, REPO1, &t)
        .expect("gh::client::delete_milestone() returned an error");

    let xs2 = c.list_milestones(ORG, REPO1)
        .expect("gh::client::list_milestones() returned an error");
    assert!(xs2.len() == 0);
}


fn random_title() -> String {
    let mut rng = rand::thread_rng();
    format!("{}-{}-{}-{}", rng.gen::<i32>(), rng.gen::<u32>(), rng.gen::<u32>(), rng.gen::<u32>())
}
