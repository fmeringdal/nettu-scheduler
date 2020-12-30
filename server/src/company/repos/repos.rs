use crate::company::domain::Company;

use std::error::Error;

#[async_trait::async_trait]
pub trait ICompanyRepo: Send + Sync {
    async fn insert(&self, company: &Company) -> Result<(), Box<dyn Error>>;
    async fn save(&self, company: &Company) -> Result<(), Box<dyn Error>>;
    async fn find(&self, company_id: &str) -> Option<Company>;
    async fn delete(&self, company_id: &str) -> Option<Company>;
}
