use super::mongo_repo::MongoMetadata;
use nettu_scheduler_domain::ID;

#[derive(Clone)]
pub struct MetadataFindQuery {
    pub metadata: MongoMetadata,
    pub skip: usize,
    pub limit: usize,
    pub account_id: ID,
}
