use crate::repos::shared::repo::DeleteResult;
use nettu_scheduler_domain::{Entity, Meta, ID};
use std::sync::Mutex;

use super::query_structs::MetadataFindQuery;

/// Useful functions for creating inmemory repositories

pub fn insert<T: Clone>(val: &T, collection: &Mutex<Vec<T>>) {
    let mut collection = collection.lock().unwrap();
    collection.push(val.clone());
}

pub fn save<T: Clone + Entity + std::fmt::Debug>(val: &T, collection: &Mutex<Vec<T>>) {
    let mut collection = collection.lock().unwrap();
    for i in 0..collection.len() {
        if collection[i].id() == val.id() {
            collection.splice(i..i + 1, vec![val.clone()]);
        }
    }
}

pub fn find<T: Clone + Entity>(val_id: &ID, collection: &Mutex<Vec<T>>) -> Option<T> {
    let collection = collection.lock().unwrap();
    for i in 0..collection.len() {
        if collection[i].id() == val_id {
            return Some(collection[i].clone());
        }
    }
    None
}

pub fn find_by<T: Clone + Entity, F: FnMut(&T) -> bool>(
    collection: &Mutex<Vec<T>>,
    mut compare: F,
) -> Vec<T> {
    let collection = collection.lock().unwrap();
    let mut items = Vec::new();
    for item in collection.iter() {
        if compare(item) {
            items.push(item.clone());
        }
    }
    items
}

pub fn delete<T: Clone + Entity>(val_id: &ID, collection: &Mutex<Vec<T>>) -> Option<T> {
    let mut collection = collection.lock().unwrap();
    for i in 0..collection.len() {
        if collection[i].id() == val_id {
            let deleted_val = collection.remove(i);
            return Some(deleted_val);
        }
    }
    None
}

pub fn delete_by<T: Clone + Entity, F: Fn(&T) -> bool>(
    collection: &Mutex<Vec<T>>,
    compare: F,
) -> DeleteResult {
    DeleteResult {
        deleted_count: find_and_delete_by(collection, compare).len() as i64,
    }
}

pub fn find_and_delete_by<T: Clone + Entity, F: Fn(&T) -> bool>(
    collection: &Mutex<Vec<T>>,
    compare: F,
) -> Vec<T> {
    let mut collection = collection.lock().unwrap();
    let mut deleted_items = Vec::new();

    for i in (0..collection.len()).rev() {
        let index = collection.len() - i - 1;
        if compare(&collection[index]) {
            let deleted_item = collection.remove(index);
            deleted_items.push(deleted_item);
        }
    }

    deleted_items
}

pub fn update_many<T: Clone + Entity, F: Fn(&T) -> bool, U: Fn(&mut T)>(
    collection: &Mutex<Vec<T>>,
    compare: F,
    update: U,
) {
    let mut collection = collection.lock().unwrap();

    for i in 0..collection.len() {
        let index = collection.len() - i - 1;
        if compare(&collection[index]) {
            update(&mut collection[index]);
        }
    }
}

/// Ignores skip and limit as this is just used for testing
pub fn find_by_metadata<T: Clone + Entity + Meta>(
    collection: &Mutex<Vec<T>>,
    query: MetadataFindQuery,
) -> Vec<T> {
    let skip = query.skip;
    let mut skipped = 0;
    let limit = query.limit;
    let mut count = 0;
    find_by(collection, |e| {
        match e.metadata().get(&query.metadata.key) {
            Some(value)
                if *value == query.metadata.value && *e.account_id() == query.account_id =>
            {
                if skip > skipped {
                    skipped += 1;
                    false
                } else if count == limit {
                    false
                } else {
                    count += 1;
                    true
                }
            }
            _ => false,
        }
    })
}
