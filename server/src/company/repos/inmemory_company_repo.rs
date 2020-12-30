use super::ICompanyRepo;
use crate::company::domain::Company;
use crate::shared::inmemory_repo::*;
use std::error::Error;

pub struct InMemoryCompanyRepo {
    companies: std::sync::Mutex<Vec<Company>>,
}

impl InMemoryCompanyRepo {
    pub fn new() -> Self {
        Self {
            companies: std::sync::Mutex::new(vec![]),
        }
    }
}

#[async_trait::async_trait]
impl ICompanyRepo for InMemoryCompanyRepo {
    async fn insert(&self, company: &Company) -> Result<(), Box<dyn Error>> {
        insert(company, &self.companies);
        Ok(())
    }

    async fn save(&self, company: &Company) -> Result<(), Box<dyn Error>> {
        save(company, &self.companies);
        Ok(())
    }

    async fn find(&self, company_id: &str) -> Option<Company> {
        find(company_id, &self.companies)
    }

    async fn delete(&self, company_id: &str) -> Option<Company> {
        delete(company_id, &self.companies)
    }
}
