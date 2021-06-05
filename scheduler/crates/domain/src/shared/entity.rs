use mongodb::bson::oid::ObjectId;
use serde::{de::Visitor, Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use thiserror::Error;

pub trait Entity<T: PartialEq> {
    fn id(&self) -> T;
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

#[derive(Debug, Clone)]
pub struct ID(ObjectId);
impl ID {
    pub fn new() -> Self {
        Self(ObjectId::new())
    }

    pub fn from(oid: ObjectId) -> Self {
        Self(oid)
    }

    pub fn as_string(&self) -> String {
        self.0.to_string()
    }

    pub fn inner(self) -> ObjectId {
        self.0
    }

    pub fn inner_ref(&self) -> &ObjectId {
        &self.0
    }
}

impl Default for ID {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

#[derive(Error, Debug)]
pub enum InvalidIDError {
    #[error("ID: {0} is malformed")]
    Malformed(String),
}

impl FromStr for ID {
    type Err = InvalidIDError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ObjectId::with_string(s)
            .map(Self)
            .map_err(|_| InvalidIDError::Malformed(s.to_string()))
    }
}

impl PartialEq for ID {
    fn eq(&self, other: &Self) -> bool {
        self.as_string() == other.as_string()
    }
}

impl Serialize for ID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.as_string())
    }
}

impl<'de> Deserialize<'de> for ID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IDVisitor;

        impl<'de> Visitor<'de> for IDVisitor {
            type Value = ID;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A valid string id representation")
            }

            fn visit_str<E>(self, value: &str) -> Result<ID, E>
            where
                E: serde::de::Error,
            {
                value
                    .parse::<ID>()
                    .map_err(|_| E::custom(format!("Malformed id: {}", value)))
            }
        }

        deserializer.deserialize_str(IDVisitor)
    }
}
