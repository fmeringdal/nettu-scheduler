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

impl KVMetadata {
    pub fn new(meta: Metadata) -> Vec<Self> {
        let mut mongo_meta = Vec::with_capacity(meta.len());
        for (key, value) in meta {
            mongo_meta.push(Self { key, value });
        }
        mongo_meta
    }

    pub fn to_metadata(mongo_metas: Vec<Self>) -> Metadata {
        let mut metadata = Metadata::with_capacity(mongo_metas.len());

        for mongo_meta in mongo_metas {
            metadata.insert(mongo_meta.key, mongo_meta.value);
        }

        metadata
    }
}
