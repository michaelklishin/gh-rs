pub type RepoName = String;

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Repo {
    id: u32,
    name: RepoName
}
