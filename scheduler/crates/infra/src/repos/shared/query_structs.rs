use nettu_scheduler_domain::{Metadata, ID};
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

pub fn extract_metadata(entries: Vec<String>) -> Metadata {
    entries
        .into_iter()
        .map(|row| {
            let key_value = row.splitn(2, '_').collect::<Vec<_>>();
            (key_value[0].to_string(), key_value[1].to_string())
        })
        .collect()
}

pub fn to_metadata(metadata: Metadata) -> Vec<String> {
    metadata
        .into_iter()
        .map(|row| format!("{}_{}", row.0, row.1))
        .collect()
}
