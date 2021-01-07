use crate::event::repos::DeleteResult;

use super::entity::Entity;
/// Useful functions for creating inmemory repositories
use std::sync::Mutex;

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

pub fn find<T: Clone + Entity>(val_id: &str, collection: &Mutex<Vec<T>>) -> Option<T> {
    let collection = collection.lock().unwrap();
    for i in 0..collection.len() {
        if collection[i].id() == val_id {
            return Some(collection[i].clone());
        }
    }
    None
}

pub fn find_by<T: Clone + Entity, F: Fn(&T) -> bool>(
    collection: &Mutex<Vec<T>>,
    compare: F,
) -> Vec<T> {
    let collection = collection.lock().unwrap();
    let mut items = vec![];
    for item in collection.iter() {
        if compare(item) {
            items.push(item.clone());
        }
    }
    items
}

pub fn delete<T: Clone + Entity>(val_id: &str, collection: &Mutex<Vec<T>>) -> Option<T> {
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
    let mut collection = collection.lock().unwrap();
    let mut deleted_count = 0;

    for i in 0..collection.len() {
        let index = collection.len() - i - 1;
        if compare(&collection[index]) {
            collection.remove(index);
            deleted_count += 1;
        }
    }

    DeleteResult { deleted_count }
}
