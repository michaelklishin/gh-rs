pub type Username = String;

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct User {
    pub id: u32,
    pub login: Username
}
