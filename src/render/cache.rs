#![allow(dead_code)]

use std::{collections::HashMap, hash::Hash};
pub struct Cache<K, V>
where
    K: Eq + Hash 
{
    hashmap: HashMap<K, V>,
}

impl<K, V> Cache<K, V> 
where 
    K: Eq + Hash
{
    pub fn new() -> Self {
        Self {
            hashmap: HashMap::new(),
        }
    }

    /// add val with key to cache
    /// return : replaced flag
    /// if new key, insert value withkey and return false
    /// if key exists, exchage val and return true
    pub fn add(&mut self, key: K, val: V) -> bool {
        let prev = self.hashmap.insert(key, val);
        matches!(prev, Some(_))
    } 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_cache() {
        let mut cache = Cache::new();
        let exists = cache.add(1, "1".to_string());
        assert_eq!(exists, false);

        let exists = cache.add(1, "1".to_string());
        assert_eq!(exists, true);

        let exists = cache.add(2, "2".to_string());
        assert_eq!(exists, false);
    }
}