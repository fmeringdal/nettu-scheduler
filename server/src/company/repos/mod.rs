mod inmemory_company_repo;
mod mongo_company_repo;
mod repos;

pub use inmemory_company_repo::InMemoryCompanyRepo;
pub use mongo_company_repo::CompanyRepo;
pub use repos::ICompanyRepo;
