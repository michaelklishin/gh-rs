use std::fmt;
use super::serde;
use super::serde::de;
use super::serde::de::{Visitor};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Open,
    Closed
}

impl serde::Serialize for State {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        match *self {
            State::Open   => ser.serialize_str("open"),
            State::Closed => ser.serialize_str("closed")
        }
    }
}

impl<'de> serde::Deserialize<'de> for State {
    fn deserialize<D>(de: D) -> Result<State, D::Error>
        where D: serde::Deserializer<'de>
    {
        struct StateVisitor;

        impl<'de> Visitor<'de> for StateVisitor {
            type Value = State;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("'open' or 'closed'")
            }

            fn visit_str<E>(self, value: &str) -> Result<State, E>
                where E: de::Error
            {
                match value {
                    "open"   => Ok(State::Open),
                    "closed" => Ok(State::Closed),
                    _        => Err(de::Error::unknown_variant(value, &["open", "closed"]))
                }
            }
        }

        de.deserialize_identifier(StateVisitor)
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let val = match *self {
            State::Open   => "open",
            State::Closed => "closed"
        };

        write!(f, "{}", val)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Milestone {
    pub id: u32,
    pub number: u32,
    pub url: String,
    pub title: String,
    pub state: State,
    // TODO: convert to a date
    pub closed_at: Option<String>
}

#[derive(Serialize, Debug)]
pub struct MilestoneProperties {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<State>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>
}

#[derive(Serialize, Debug)]
pub struct MilestonePatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<State>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<String>
}
