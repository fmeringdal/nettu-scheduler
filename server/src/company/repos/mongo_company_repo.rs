use crate::company::domain::Company;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, from_bson, oid::ObjectId, Bson, Document},
    Collection, Database,
};
use std::error::Error;
use tokio::sync::RwLock;

use super::ICompanyRepo;

pub struct CompanyRepo {
    collection: RwLock<Collection>,
}

// RwLock is Send + Sync
unsafe impl Send for CompanyRepo {}
unsafe impl Sync for CompanyRepo {}

impl CompanyRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: RwLock::new(db.collection("companies")),
        }
    }
}

#[async_trait::async_trait]
impl ICompanyRepo for CompanyRepo {
    async fn insert(&self, company: &Company) -> Result<(), Box<dyn Error>> {
        let coll = self.collection.read().await;
        let _res = coll.insert_one(to_persistence(company), None).await;
        Ok(())
    }

    async fn save(&self, company: &Company) -> Result<(), Box<dyn Error>> {
        let coll = self.collection.read().await;
        let filter = doc! {
            "_id": ObjectId::with_string(&company.id)?
        };
        let _res = coll
            .update_one(filter, to_persistence(company), None)
            .await;
        Ok(())
    }

    async fn find(&self, company_id: &str) -> Option<Company> {
        let filter = doc! {
            "_id": ObjectId::with_string(company_id).unwrap()
        };
        let coll = self.collection.read().await;
        let res = coll.find_one(filter, None).await;
        match res {
            Ok(doc) if doc.is_some() => {
                let company = to_domain(doc.unwrap());
                Some(company)
            }
            _ => None,
        }
    }

    async fn delete(&self, company_id: &str) -> Option<Company> {
        let filter = doc! {
            "_id": ObjectId::with_string(company_id).unwrap()
        };
        let coll = self.collection.read().await;
        let res = coll.find_one_and_delete(filter, None).await;
        match res {
            Ok(doc) if doc.is_some() => {
                let company = to_domain(doc.unwrap());
                Some(company)
            }
            _ => None,
        }
    }
}

fn to_persistence(company: &Company) -> Document {
    let raw = doc! {
        "_id": ObjectId::with_string(&company.id).unwrap()
    };

    raw
}

fn to_domain(raw: Document) -> Company {
    let id = match raw.get("_id").unwrap() {
        Bson::ObjectId(oid) => oid.to_string(),
        _ => unreachable!("This should not happen"),
    };

    let company = Company {
        id,
    };

    company
}
