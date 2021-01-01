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
