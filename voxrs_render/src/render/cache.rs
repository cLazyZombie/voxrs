use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

pub struct Cache<K, V>
where
    K: Eq + Hash + Copy,
{
    cached: HashMap<K, Vec<V>>,
    used: HashSet<K>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Copy,
{
    pub fn new() -> Self {
        Self {
            cached: HashMap::new(),
            used: HashSet::new(),
        }
    }

    /// add val vec with key to cache
    /// return : replaced flag
    /// if new key, insert value withkey and return false
    /// if key exists, exchage val and return true
    pub fn add(&mut self, key: K, vec: Vec<V>) -> bool {
        let prev = self.cached.insert(key, vec);
        matches!(prev, Some(_))
    }

    /// refresh used and return alreay used
    ///
    /// return true if already used
    /// retgurn false if new key
    pub fn refresh(&mut self, key: K) -> bool {
        !self.used.insert(key)
    }

    pub fn get(&self, key: &K) -> Option<impl Iterator<Item = &V>> {
        let value = self.cached.get(key);
        match value {
            Some(v) => Some(v.iter()),
            None => None,
        }
    }

    pub fn clear_unused(&mut self) -> usize {
        let cached: HashSet<_> = self.cached.iter().map(|(k, _)| *k).collect();
        let removed: Vec<_> = cached.difference(&self.used).collect();

        for &k in &removed {
            self.cached.remove(k);
        }

        let remove_count = removed.len();

        self.used.clear();

        remove_count
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;

    #[test]
    fn clear_unused() {
        let mut cache = Cache::new();

        assert_eq!(cache.refresh(1), false);
        cache.add(1, vec!["1".to_string()]);

        assert_eq!(cache.refresh(2), false);
        cache.add(2, vec!["2".to_string()]);

        let removed = cache.clear_unused();
        assert_eq!(removed, 0);

        cache.refresh(2);

        let removed = cache.clear_unused();
        assert_eq!(removed, 1);

        assert!(cache.get(&1).is_none());
        assert!(cache.get(&2).is_some());
    }

    #[test]
    fn test_get() {
        let mut cache = Cache::new();
        cache.add(1, vec!["1".to_string()]);

        let mut cached = cache.get(&1).unwrap();
        assert_eq!(cached.next(), Some(&"1".to_string()));
    }
}
