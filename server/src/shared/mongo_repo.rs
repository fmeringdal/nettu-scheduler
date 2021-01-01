use anyhow::Result;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection,
};
use std::error::Error;
use tokio::sync::RwLock;

pub enum MongoPersistenceID {
    ObjectId(ObjectId),
    String(String),
}

pub trait MongoPersistence {
    fn to_domain(doc: Document) -> Self;
    fn to_persistence(&self) -> Document;
    fn get_persistence_id(&self) -> Result<MongoPersistenceID>;
}

fn get_id_filter<T: MongoPersistence>(val_id: &MongoPersistenceID) -> Document {
    match val_id {
        MongoPersistenceID::ObjectId(oid) => doc! {
            "_id": oid
        },
        MongoPersistenceID::String(id) => doc! {
            "_id": id
        },
    }
}

pub async fn insert<T: MongoPersistence>(collection: &RwLock<Collection>, val: &T) -> Result<()> {
    let coll = collection.read().await;
    let _res = coll.insert_one(val.to_persistence(), None).await;
    Ok(())
}

pub async fn save<T: MongoPersistence>(collection: &RwLock<Collection>, val: &T) -> Result<()> {
    let coll = collection.read().await;
    let val_id = val.get_persistence_id()?;
    let filter = get_id_filter::<T>(&val_id);
    let _res = coll.update_one(filter, val.to_persistence(), None).await;
    Ok(())
}

pub async fn find<T: MongoPersistence>(
    collection: &RwLock<Collection>,
    id: &MongoPersistenceID,
) -> Option<T> {
    let filter = get_id_filter::<T>(id);
    find_one_by(collection, filter).await
}

pub async fn find_one_by<T: MongoPersistence>(
    collection: &RwLock<Collection>,
    filter: Document,
) -> Option<T> {
    let coll = collection.read().await;
    let res = coll.find_one(filter, None).await;
    match res {
        Ok(doc) if doc.is_some() => {
            let event = T::to_domain(doc.unwrap());
            Some(event)
        }
        _ => None,
    }
}

pub async fn find_many_by<T: MongoPersistence>(
    collection: &RwLock<Collection>,
    filter: Document,
) -> Result<Vec<T>, Box<dyn Error>> {
    let coll = collection.read().await;
    let res = coll.find(filter, None).await;

    match res {
        Ok(mut cursor) => {
            let mut documents = vec![];
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        documents.push(T::to_domain(document));
                    }
                    Err(e) => {
                        println!("Error getting cursor calendar event: {:?}", e);
                    }
                }
            }

            Ok(documents)
        }
        Err(err) => Err(Box::new(err)),
    }
}

pub async fn delete<T: MongoPersistence>(
    collection: &RwLock<Collection>,
    id: &MongoPersistenceID,
) -> Option<T> {
    let filter = get_id_filter::<T>(id);
    let coll = collection.read().await;
    let res = coll.find_one_and_delete(filter, None).await;
    match res {
        Ok(doc) if doc.is_some() => {
            let event = T::to_domain(doc.unwrap());
            Some(event)
        }
        _ => None,
    }
}
