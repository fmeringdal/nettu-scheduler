use crate::{Entity, ID};
use std::collections::HashMap;

pub type Metadata = HashMap<String, String>;

pub trait Meta<T: PartialEq>: Entity<T> {
    fn metadata(&self) -> &Metadata;
    /// Retrieves the account_id associated with this entity, which
    /// is useful to know when querying on the metadata
    fn account_id(&self) -> &ID;
}
