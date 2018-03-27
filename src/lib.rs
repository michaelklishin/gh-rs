extern crate reqwest;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

extern crate failure;
#[macro_use]
extern crate failure_derive;

use reqwest::header;

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

#[derive(Debug, Fail)]
pub enum ResponseError {
    #[fail(display = "API responded with 404 Not Found")]
    NotFound,

    #[fail(display = "HTTP request resulted in an error: {}", _0)]
    HTTPError(#[cause] reqwest::Error),

    #[fail(display = "JSON serialization failed: {}", _0)]
    SerializationError(#[cause] serde_json::error::Error)
}

impl From<reqwest::Error> for ResponseError {
    fn from(error: reqwest::Error) -> Self {
        ResponseError::HTTPError(error)
    }
}

impl From<serde_json::error::Error> for ResponseError {
    fn from(error: serde_json::error::Error) -> Self {
        ResponseError::SerializationError(error)
    }
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

    pub fn current_user(&self) -> Result<users::User, ResponseError> {
        let path = format!("{}/{}", API_BASE, "user");
        let mut res = self.http_client
            .get(&path)
            .send()?;

        let payload: users::User = res.json()?;
        Ok(payload)
    }

    pub fn list_repos_of_org(&self, org: &str) -> Result<Vec<repos::Repo>, ResponseError> {
        let path = format!("{}/orgs/{}/repos", API_BASE, org);
        let mut res = self.http_client
            .get(&path)
            .send()?;

        let payload: Vec<repos::Repo> = res.json()?;
        Ok(payload)
    }

    pub fn list_milestones(&self, user: &str, repo: &str) -> Result<Vec<milestones::Milestone>, ResponseError> {
        let path = format!("{}/repos/{}/{}/milestones", API_BASE, user, repo);
        let mut res = self.http_client
            .get(&path)
            .send()?;

        let payload: Vec<milestones::Milestone> = res.json()?;
        Ok(payload)
    }

    pub fn get_milestone(&self, user: &str, repo: &str, number: u32) -> Result<milestones::Milestone, ResponseError> {
        let path = format!("{}/repos/{}/{}/milestones/{}", API_BASE, user, repo, number);
        let mut res = self.http_client
            .get(&path)
            .send()?;

        let payload: milestones::Milestone = res.json()?;
        Ok(payload)
    }

    pub fn open_milestone(&self, user: &str, repo: &str, number: u32) -> Result<milestones::Milestone, ResponseError> {
        self.update_milestone_state(user, repo, number, &milestones::State::Open)
    }

    pub fn get_milestone_with_title(&self, user: &str, repo: &str, title: &str)
                                    -> Result<milestones::Milestone, ResponseError> {
        let milestones = self.list_milestones(user, repo)?;

        let found = milestones.iter()
            .find(|&m| m.title == title)
            .map(|v| v.clone());
        
        found.ok_or(ResponseError::NotFound)
    }

    pub fn close_milestone(&self, user: &str, repo: &str, title: &str) -> Result<milestones::Milestone, ResponseError> {
        let m = self.get_milestone_with_title(user, repo, title)?;

        self.update_milestone_state(user, repo, m.number, &milestones::State::Closed)
    }

    pub fn create_milestone(&self, user: &str, repo: &str, props: &milestones::MilestoneProperties)
                            -> Result<milestones::Milestone, ResponseError> {
        let path = format!("{}/repos/{}/{}/milestones", API_BASE, user, repo);

        let mut res = self.http_client
            .post(&path)
            .json(&props)
            .send()?;

        let payload: milestones::Milestone = res.json()?;
        Ok(payload)
    }

    pub fn update_milestone(&self, user: &str, repo: &str, number: u32, patch: &milestones::MilestonePatch)
                            -> Result<milestones::Milestone, ResponseError> {
        let path = format!("{}/repos/{}/{}/milestones/{}", API_BASE, user, repo, number);

        let mut res = self.http_client
            .patch(&path)
            .json(&patch)
            .send()?;

        let payload: milestones::Milestone = res.json()?;
        Ok(payload)
    }

    fn update_milestone_state(&self, user: &str, repo: &str, number: u32, state: &milestones::State)
                              -> Result<milestones::Milestone, ResponseError> {
        let hm = milestones::MilestonePatch {
            title: None,
            state: Some(state.clone()),
            description: None,
            due_on: None
        };

        self.update_milestone(user, repo, number, &hm)
    }

    pub fn delete_milestone_with_title(&self, user: &str, repo: &str, title: &str) -> Result<(), ResponseError> {
        let m = self.get_milestone_with_title(user, repo, title)?;

        self.delete_milestone(user, repo, m.number)
    }

    pub fn delete_milestone(&self, user: &str, repo: &str, number: u32) -> Result<(), ResponseError> {
        let path = format!("{}/repos/{}/{}/milestones/{}", API_BASE, user, repo, number);

        self.http_client
            .delete(&path)
            .send()?;
        Ok(())
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
