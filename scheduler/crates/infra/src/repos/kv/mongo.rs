use super::IKVRepo;
use crate::repos::shared::mongo_repo::MongoDocument;
use crate::repos::shared::{mongo_repo, query_structs::MetadataFindQuery};
use crate::KVMetadata;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};
use nettu_scheduler_domain::{User, UserIntegrationProvider, ID};
use serde::{Deserialize, Serialize};

pub struct MongoKVRepo {
    collection: Collection,
}

impl MongoKVRepo {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("key-values"),
        }
    }
}

#[async_trait::async_trait]
impl IKVRepo for MongoKVRepo {
    async fn insert(&self, user: &User) -> anyhow::Result<()> {
        mongo_repo::insert::<_, UserMongo>(&self.collection, user).await
    }

    async fn save(&self, user: &User) -> anyhow::Result<()> {
        mongo_repo::save::<_, UserMongo>(&self.collection, user).await
    }

    async fn find_by_metadata(&self, query: MetadataFindQuery) -> Vec<User> {
        mongo_repo::find_by_metadata::<_, UserMongo>(&self.collection, query).await
    }

    async fn find(&self, user_id: &ID) -> Option<User> {
        let oid = user_id.inner_ref();
        mongo_repo::find::<_, UserMongo>(&self.collection, &oid).await
    }

    async fn delete(&self, user_id: &ID) -> Option<User> {
        let oid = user_id.inner_ref();
        mongo_repo::delete::<_, UserMongo>(&self.collection, &oid).await
    }

    async fn find_by_account_id(&self, user_id: &ID, account_id: &ID) -> Option<User> {
        let filter = doc! {
            "_id": user_id.inner_ref(),
            "account_id": account_id.inner_ref()
        };
        mongo_repo::find_one_by::<_, UserMongo>(&self.collection, filter).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct UserMongo {
    _id: ObjectId,
    account_id: ObjectId,
    metadata: Vec<KVMetadata>,
    integrations: Vec<UserIntegrationProvider>,
}

impl MongoDocument<User> for UserMongo {
    fn to_domain(self) -> User {
        User {
            id: ID::from(self._id),
            account_id: ID::from(self.account_id),
            metadata: KVMetadata::to_metadata(self.metadata),
            integrations: self.integrations,
        }
    }

    fn from_domain(user: &User) -> Self {
        Self {
            _id: user.id.inner_ref().clone(),
            account_id: user.account_id.inner_ref().clone(),
            metadata: KVMetadata::new(user.metadata.clone()),
            integrations: user.integrations.clone(),
        }
    }

    fn get_id_filter(&self) -> Document {
        doc! {
            "_id": &self._id
        }
    }
}
