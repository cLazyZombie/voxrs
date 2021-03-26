use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    hash::Hash,
};

pub struct Cache<K, V>
where
    K: Eq + Hash + Copy,
{
    cached: HashMap<K, Vec<V>>,
    used: RefCell<HashSet<K>>,
}

impl<K, V> Cache<K, V>
where
    K: Eq + Hash + Copy,
{
    pub fn new() -> Self {
        Self {
            cached: HashMap::new(),
            used: RefCell::new(HashSet::new()),
        }
    }

    /// add val vec with key to cache
    /// return : replaced flag
    /// if new key, insert value withkey and return false
    /// if key exists, exchange val and return true
    pub fn add(&mut self, key: K, vec: Vec<V>) -> bool {
        let prev = self.cached.insert(key, vec);
        self.set_used(key);
        matches!(prev, Some(_))
    }

    /// refresh used and return alreay used
    ///
    /// return true if already used
    /// retgurn false if new key
    pub fn set_used(&self, key: K) -> bool {
        !self.used.borrow_mut().insert(key)
    }

    // pub fn try_iter(&self, key: &K) -> Option<impl Iterator<Item = &V>> {
    //     self.set_used(*key);

    //     let value = self.cached.get(key);
    //     match value {
    //         Some(v) => Some(v.iter()),
    //         None => None,
    //     }
    // }

    pub fn get(&self, key: &K) -> Option<&Vec<V>> {
        self.set_used(*key);

        let value = self.cached.get(key);
        match value {
            Some(v) => Some(v),
            None => None,
        }
    }

    pub fn clear_unused(&mut self) -> usize {
        let remove_count;
        {
            let cached: HashSet<_> = self.cached.iter().map(|(k, _)| *k).collect();
            let used = &self.used.borrow();
            let removed: Vec<_> = cached.difference(&used).collect();
            remove_count = removed.len();

            for &k in &removed {
                self.cached.remove(k);
            }
        }

        self.used.get_mut().clear();
        remove_count
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;

    #[test]
    fn clear_unused() {
        let mut cache = Cache::new();

        assert_eq!(cache.set_used(1), false);
        cache.add(1, vec!["1".to_string()]);

        assert_eq!(cache.set_used(2), false);
        cache.add(2, vec!["2".to_string()]);

        let removed = cache.clear_unused();
        assert_eq!(removed, 0);

        cache.set_used(2);

        let removed = cache.clear_unused();
        assert_eq!(removed, 1);

        assert!(cache.get(&1).is_none());
        assert!(cache.get(&2).is_some());
    }

    #[test]
    fn test_get() {
        let mut cache = Cache::new();
        cache.add(1, vec!["1".to_string()]);

        let cached = cache.get(&1).unwrap();
        assert_eq!(cached.iter().next(), Some(&"1".to_string()));
    }
}
