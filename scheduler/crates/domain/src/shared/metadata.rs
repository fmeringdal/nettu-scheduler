use crate::{Entity, ID};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Metadata {
    #[serde(flatten)]
    pub inner: HashMap<String, String>,
}

impl Metadata {
    pub fn new(inner: HashMap<String, String>) -> Self {
        Self { inner }
    }
}

pub trait Meta<T: PartialEq>: Entity<T> {
    fn metadata(&self) -> &Metadata;
    /// Retrieves the account_id associated with this entity, which
    /// is useful to know when querying on the metadata
    fn account_id(&self) -> &ID;
}
