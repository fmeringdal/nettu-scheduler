use crate::repos::shared::repo::DeleteResult;
use nettu_scheduler_domain::Entity;
use std::sync::Mutex;

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
    DeleteResult {
        deleted_count: find_and_delete_by(collection, compare).len() as i64,
    }
}

pub fn find_and_delete_by<T: Clone + Entity, F: Fn(&T) -> bool>(
    collection: &Mutex<Vec<T>>,
    compare: F,
) -> Vec<T> {
    let mut collection = collection.lock().unwrap();
    let mut deleted_items = vec![];

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
