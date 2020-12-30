use super::ICompanyRepo;
use crate::company::domain::Company;
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
        let mut companies = self.companies.lock().unwrap();
        companies.push(company.clone());
        Ok(())
    }

    async fn save(&self, company: &Company) -> Result<(), Box<dyn Error>> {
        let mut companies = self.companies.lock().unwrap();
        for i in 0..companies.len() {
            if companies[i].id == company.id {
                companies.splice(i..i + 1, vec![company.clone()]);
            }
        }
        Ok(())
    }

    async fn find(&self, company_id: &str) -> Option<Company> {
        let companies = self.companies.lock().unwrap();
        for i in 0..companies.len() {
            if companies[i].id == company_id {
                return Some(companies[i].clone());
            }
        }
        None
    }

    async fn delete(&self, company_id: &str) -> Option<Company> {
        let mut companies = self.companies.lock().unwrap();
        for i in 0..companies.len() {
            if companies[i].id == company_id {
                let deleted_company = companies.remove(i);
                return Some(deleted_company);
            }
        }
        None
    }
}
