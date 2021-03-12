use std::cmp::max;

use super::{query_structs::MetadataFindQuery, repo::DeleteResult};
use anyhow::Result;
use futures::stream::StreamExt;
use mongodb::{
    bson::{self, doc, oid::ObjectId, to_bson, Document},
    options::FindOptions,
    Collection, Cursor,
};

use serde::{de::DeserializeOwned, Serialize};
use tracing::error;

pub trait MongoDocument<E>: Serialize + DeserializeOwned {
    fn to_domain(self) -> E;
    fn from_domain(entity: &E) -> Self;
    fn get_id_filter(&self) -> Document;
}

fn get_id_filter(oid: &ObjectId) -> Document {
    doc! {
        "_id": oid
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

pub async fn bulk_insert<E, D: MongoDocument<E>>(
    collection: &Collection,
    entities: &[E],
) -> Result<()> {
    let docs = entities
        .iter()
        .map(|e| entity_to_persistence::<E, D>(e))
        .collect::<Vec<_>>();
    let _res = collection.insert_many(docs, None).await;
    Ok(())
}

pub async fn save<E, D: MongoDocument<E>>(collection: &Collection, entity: &E) -> Result<()> {
    let raw = D::from_domain(entity);
    let filter = raw.get_id_filter();
    let doc = doc_to_persistence(&raw);
    let _res = collection.update_one(filter, doc, None).await;
    Ok(())
}

pub async fn update_many<E, D: MongoDocument<E>>(
    collection: &Collection,
    filter: Document,
    update: Document,
) -> Result<()> {
    let coll = collection;
    coll.update_many(filter, update, None)
        .await
        .map(|_| ())
        .map_err(anyhow::Error::new)
}

pub async fn find<E, D: MongoDocument<E>>(collection: &Collection, id: &ObjectId) -> Option<E> {
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
) -> Result<Vec<E>> {
    let coll = collection;
    let res = coll.find(filter, None).await;

    match res {
        Ok(cursor) => Ok(consume_cursor::<E, D>(cursor).await),
        Err(err) => Err(anyhow::Error::new(err)),
    }
}

pub async fn delete<E, D: MongoDocument<E>>(collection: &Collection, id: &ObjectId) -> Option<E> {
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

pub async fn delete_many_by<E, D: MongoDocument<E>>(
    collection: &Collection,
    filter: Document,
) -> Result<DeleteResult> {
    let res = collection.delete_many(filter, None).await?;
    Ok(DeleteResult {
        deleted_count: res.deleted_count,
    })
}

async fn consume_cursor<E, D: MongoDocument<E>>(mut cursor: Cursor) -> Vec<E> {
    let mut documents = vec![];
    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                documents.push(persistence_to_entity::<E, D>(document));
            }
            Err(e) => {
                error!("Error getting cursor for calendar event repo: {:?}", e);
            }
        }
    }

    documents
}

pub async fn find_by_metadata<E, D: MongoDocument<E>>(
    collection: &Collection,
    query: MetadataFindQuery,
) -> Vec<E> {
    let limit = max(query.limit, 100);

    let filter = doc! {
        "metadata": {
            "$elemMatch": {
                "key": query.metadata.key,
                "value": query.metadata.value
            }
        },
        "account_id": query.account_id.inner()
    };

    let mut find_options = FindOptions::builder().build();
    find_options.skip = Some(query.skip as i64);
    find_options.limit = Some(limit as i64);

    // find_options.
    match collection.find(filter, find_options).await {
        Ok(cursor) => consume_cursor::<E, D>(cursor).await,
        Err(_) => vec![],
    }
}
