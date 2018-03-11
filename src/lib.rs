extern crate reqwest;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate error_chain;

use reqwest::header;

mod errors {
    error_chain! {
        errors {
            NotFound {
                description("API responded with 404 Not Found")
                display("404 Not Found")
            }

            HTTPError {
                description("HTTP request error")
                display("HTTP request resulted in an error")
            }

            SerializationError {
                description("[de]serialization error")
                display("JSON serialization failed")
            }

        }
    }
}

use errors::*;

pub mod milestones;
pub mod repos;
pub mod users;

const USER_AGENT: &str = "github.com/michaelklishin/gh-rs";
const API_BASE: &str = "https://api.github.com";

pub fn client(token: &str) -> Client {
    Client::new(token)
}

#[derive(Debug)]
pub struct Client {
    http_client: self::reqwest::Client
}

#[derive(Debug)]
pub enum Error {

}

impl Client {

    //
    // API
    //

    pub fn new(token: &str) -> Client {
        let hs = build_default_headers(token);
        let mut builder = build_http_request_builder(hs);

        Client {
            http_client: builder.build().unwrap()
        }
    }

    pub fn current_user(&self) -> Result<users::User> {
        let path = format!("{}/{}", API_BASE, "user");
        let mut res = self.http_client
            .get(&path)
            .send()
            .chain_err(|| ErrorKind::HTTPError)?;

        let payload = &res.text()
            .chain_err(|| ErrorKind::HTTPError)?;
        serde_json::from_str::<users::User>(payload)
            .chain_err(|| ErrorKind::SerializationError)
    }

    pub fn list_repos_of_org(&self, org: &str) -> Result<Vec<repos::Repo>> {
        let path = format!("{}/orgs/{}/repos", API_BASE, org);
        let mut res = self.http_client
            .get(&path)
            .send()
            .chain_err(|| ErrorKind::HTTPError)?;

        let payload = &res.text()
            .chain_err(|| ErrorKind::HTTPError)?;
        serde_json::from_str::<Vec<repos::Repo>>(payload)
            .chain_err(|| ErrorKind::SerializationError)
    }

    pub fn list_milestones(&self, user: &str, repo: &str) -> Result<Vec<milestones::Milestone>> {
        let path = format!("{}/repos/{}/{}/milestones", API_BASE, user, repo);
        let mut res = self.http_client
            .get(&path)
            .send()
            .chain_err(|| ErrorKind::HTTPError)?;

        let payload = &res.text()
            .chain_err(|| ErrorKind::HTTPError)?;
        serde_json::from_str::<Vec<milestones::Milestone>>(payload)
            .chain_err(|| ErrorKind::SerializationError)
    }

    pub fn get_milestone(&self, user: &str, repo: &str, number: u32) -> Result<milestones::Milestone> {
        let path = format!("{}/repos/{}/{}/milestones/{}", API_BASE, user, repo, number);
        let mut res = self.http_client
            .get(&path)
            .send()
            .chain_err(|| ErrorKind::HTTPError)?;

        let payload = &res.text()
            .chain_err(|| ErrorKind::HTTPError)?;
        serde_json::from_str::<milestones::Milestone>(payload)
            .chain_err(|| ErrorKind::SerializationError)
    }

    pub fn open_milestone(&self, user: &str, repo: &str, number: u32) -> Result<milestones::Milestone> {
        self.update_milestone_state(user, repo, number, &milestones::State::Open)
    }

    pub fn get_milestone_with_title(&self, user: &str, repo: &str, title: &str)
                                    -> Result<milestones::Milestone> {
        let milestones = self.list_milestones(user, repo)?;

        milestones.iter().find(|&m| m.title == title)
            .map(|v| v.clone())
            .chain_err(|| ErrorKind::NotFound)

    }

    pub fn close_milestone(&self, user: &str, repo: &str, title: &str) -> Result<milestones::Milestone> {
        let m = self.get_milestone_with_title(user, repo, title)?;

        self.update_milestone_state(user, repo, m.number, &milestones::State::Closed)
    }

    pub fn create_milestone(&self, user: &str, repo: &str, props: &milestones::MilestoneProperties) -> Result<milestones::Milestone> {
        let path = format!("{}/repos/{}/{}/milestones", API_BASE, user, repo);

        let mut res = self.http_client
            .post(&path)
            .json(&props)
            .send()
            .chain_err(|| ErrorKind::HTTPError)?;

        let payload = &res.text()
            .chain_err(|| ErrorKind::HTTPError)?;

        serde_json::from_str::<milestones::Milestone>(payload)
            .chain_err(|| ErrorKind::SerializationError)
    }

    pub fn update_milestone(&self, user: &str, repo: &str, number: u32, patch: &milestones::MilestonePatch) -> Result<milestones::Milestone> {
        let path = format!("{}/repos/{}/{}/milestones/{}", API_BASE, user, repo, number);

        let mut res = self.http_client
            .patch(&path)
            .json(&patch)
            .send()
            .chain_err(|| ErrorKind::HTTPError)?;

        let payload = &res.text()
            .chain_err(|| ErrorKind::HTTPError)?;

        serde_json::from_str::<milestones::Milestone>(payload)
            .chain_err(|| ErrorKind::SerializationError)
    }

    fn update_milestone_state(&self, user: &str, repo: &str, number: u32, state: &milestones::State) -> Result<milestones::Milestone> {
        let hm = milestones::MilestonePatch {
            title: None,
            state: Some(state.clone()),
            description: None,
            due_on: None
        };

        self.update_milestone(user, repo, number, &hm)
    }

    pub fn delete_milestone_with_title(&self, user: &str, repo: &str, title: &str) -> Result<()> {
        let m = self.get_milestone_with_title(user, repo, title)?;

        self.delete_milestone(user, repo, m.number)
    }

    pub fn delete_milestone(&self, user: &str, repo: &str, number: u32) -> Result<()> {
        let path = format!("{}/repos/{}/{}/milestones/{}", API_BASE, user, repo, number);

        self.http_client
            .delete(&path)
            .send()
            .chain_err(|| ErrorKind::HTTPError)
            .map (|_| () )
    }
}

//
// Implementation
//

fn build_http_request_builder(hs: reqwest::header::Headers) -> reqwest::ClientBuilder {
    let mut builder = reqwest::Client::builder();
    // note: this *appends* to the default set of headers reqwest
    //       uses
    builder.default_headers(hs);
    builder
}

fn build_default_headers(token: &str) -> reqwest::header::Headers {
    let mut authorization_val: String = String::from("token ");
    authorization_val.push_str(token);

    let mut hs = header::Headers::new();
    hs.set(header::Authorization(authorization_val));
    // Per https://developer.github.com/v3/#user-agent-required.
    hs.set(header::UserAgent::new(USER_AGENT));
    hs
}
