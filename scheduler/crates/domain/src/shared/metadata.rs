use std::collections::HashMap;

use crate::{Entity, ID};

pub type Metadata = HashMap<String, String>;

pub trait Meta: Entity {
    fn metadata(&self) -> &Metadata;
    /// Retrives the account_id associated with this entity, which
    /// is useful to know when querying on the metadata
    fn account_id(&self) -> &ID;
}
