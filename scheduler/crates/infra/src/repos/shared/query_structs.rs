use nettu_scheduler_domain::{Metadata, ID};

#[derive(Debug, Clone)]
pub struct MetadataFindQuery {
    pub metadata: Metadata,
    pub skip: usize,
    pub limit: usize,
    pub account_id: ID,
}
