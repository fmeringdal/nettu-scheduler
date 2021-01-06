use anyhow::Result;
use futures::stream::StreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, to_bson, Document},
    Collection,
};
use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;

pub enum MongoPersistenceID {
    ObjectId(ObjectId),
    String(String),
}

pub trait MongoDocument<E>: Serialize + DeserializeOwned {
    fn to_domain(&self) -> E;
    fn from_domain(entity: &E) -> Self;
    fn get_id_filter(&self) -> Document;
}

fn get_id_filter(val_id: &MongoPersistenceID) -> Document {
    match val_id {
        MongoPersistenceID::ObjectId(oid) => doc! {
            "_id": oid
        },
        MongoPersistenceID::String(id) => doc! {
            "_id": id
        },
    }
}

fn entity_to_persistence<E, D: MongoDocument<E>>(entity: &E) -> Document {
    let raw = D::from_domain(entity);
    doc_to_persistence(&raw)
}

fn persistence_to_entity<E, D: MongoDocument<E>>(doc: Document) -> E {
    // let bson = bson::Bson::Document(doc);
    let raw: D = bson::from_document(doc).unwrap();
    raw.to_domain()
}

fn doc_to_persistence<E, D: MongoDocument<E>>(raw: &D) -> Document {
    to_bson(raw).unwrap().as_document().unwrap().to_owned()
}

pub async fn insert<E, D: MongoDocument<E>>(collection: &Collection, entity: &E) -> Result<()> {
    let doc = entity_to_persistence::<E, D>(entity);
    let _res = collection.insert_one(doc, None).await;
    Ok(())
}

pub async fn save<E, D: MongoDocument<E>>(collection: &Collection, entity: &E) -> Result<()> {
    let raw = D::from_domain(entity);
    let filter = raw.get_id_filter();
    let doc = doc_to_persistence(&raw);
    let _res = collection.update_one(filter, doc, None).await;
    Ok(())
}

pub async fn find<E, D: MongoDocument<E>>(
    collection: &Collection,
    id: &MongoPersistenceID,
) -> Option<E> {
    let filter = get_id_filter(id);
    find_one_by::<E, D>(collection, filter).await
}

pub async fn find_one_by<E, D: MongoDocument<E>>(
    collection: &Collection,
    filter: Document,
) -> Option<E> {
    let res = collection.find_one(filter, None).await;
    match res {
        Ok(doc) if doc.is_some() => {
            let doc = doc.unwrap();
            let e = persistence_to_entity::<E, D>(doc);
            Some(e)
        }
        _ => None,
    }
}

pub async fn find_many_by<E, D: MongoDocument<E>>(
    collection: &Collection,
    filter: Document,
) -> Result<Vec<E>, Box<dyn Error>> {
    let coll = collection;
    let res = coll.find(filter, None).await;

    match res {
        Ok(mut cursor) => {
            let mut documents = vec![];
            while let Some(result) = cursor.next().await {
                match result {
                    Ok(document) => {
                        documents.push(persistence_to_entity::<E, D>(document));
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

pub async fn delete<E, D: MongoDocument<E>>(
    collection: &Collection,
    id: &MongoPersistenceID,
) -> Option<E> {
    let filter = get_id_filter(id);
    let res = collection.find_one_and_delete(filter, None).await;
    match res {
        Ok(doc) if doc.is_some() => {
            let event = persistence_to_entity::<E, D>(doc.unwrap());
            Some(event)
        }
        _ => None,
    }
}
