use nettu_scheduler_domain::Metadata;
use nettu_scheduler_domain::ID;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct MetadataFindQuery {
    pub metadata: KVMetadata,
    pub skip: usize,
    pub limit: usize,
    pub account_id: ID,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KVMetadata {
    pub key: String,
    pub value: String,
}
