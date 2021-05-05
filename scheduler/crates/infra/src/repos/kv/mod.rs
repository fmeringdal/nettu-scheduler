mod inmemory;
mod mongo;

use std::collections::HashMap;

pub use inmemory::InMemoryKVRepo;
pub use mongo::MongoKVRepo;

use super::shared::query_structs::MetadataFindQuery;

struct KeyValue {
    pub key: String,
    pub value: String,
    pub data: HashMap<String, String>,
}

#[async_trait::async_trait]
pub trait IKVRepo: Send + Sync {
    async fn set(&self, kv: &KeyValue) -> anyhow::Result<()>;
    async fn get(&self, key: &str) -> Option<KeyValue>;
    async fn delete(&self, key: &str) -> anyhow::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{setup_context, NettuContext};

    /// Creates inmemory and mongo context when mongo is running,
    /// otherwise it will create two inmemory
    async fn create_contexts() -> Vec<NettuContext> {
        vec![NettuContext::create_inmemory(), setup_context().await]
    }

    #[tokio::test]
    async fn test_kv_queries() {
        let ctxs = create_contexts().await;

        for ctx in ctxs {
            let kv1 = KeyValue {
                key: "1".into(),
                value: "1".into(),
                data: Default::default(),
            };
            let kv2 = KeyValue {
                key: "2".into(),
                value: "2".into(),
                data: Default::default(),
            };
            let kv3 = KeyValue {
                key: kv1.key.clone(),
                value: "3".into(),
                data: Default::default(),
            };

            for kv in vec![&kv1, &kv2] {
                assert!(ctx.repos.key_value_repo.set(&kv).await.is_ok());
                let res = ctx
                    .repos
                    .key_value_repo
                    .get(&kv.key)
                    .await
                    .expect("To find key value just inserted");
                assert_eq!(res.key, kv.key);
                assert_eq!(res.value, kv.value);
            }

            // Delete kv2 key and query on that should return None
            assert!(ctx.repos.key_value_repo.delete(key))
        }
    }
}
