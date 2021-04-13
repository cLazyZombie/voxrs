use std::collections::{HashMap, HashSet};

use voxrs_types::SafeCloner;

use crate::blueprint::{self, ChunkId};

use super::chunk::Chunk;

/// should store with safe cloner for being noticed that safe cloner is changed
type CachedValue = (Vec<Chunk>, SafeCloner<blueprint::Chunk>);

pub(crate) struct ChunkCache {
    cached: HashMap<ChunkId, CachedValue>,
    used: HashSet<ChunkId>,
}

impl ChunkCache {
    pub fn new() -> Self {
        Self {
            cached: HashMap::new(),
            used: HashSet::new(),
        }
    }

    /// add val vec with key to cache
    /// return : replaced flag
    /// if new key, insert value withkey and return false
    /// if key exists, exchange val and return true
    pub fn add(
        &mut self,
        key: ChunkId,
        chunk_bp: SafeCloner<blueprint::Chunk>,
        chunks: Vec<Chunk>,
    ) -> bool {
        let prev = self.cached.insert(key, (chunks, chunk_bp));
        self.set_used(key);
        matches!(prev, Some(_))
    }

    /// refresh used and return alreay used
    ///
    /// return true if already used
    /// retgurn false if new key
    pub fn set_used(&mut self, key: ChunkId) -> bool {
        !self.used.insert(key)
    }

    // pub fn try_iter(&self, key: &K) -> Option<impl Iterator<Item = &V>> {
    //     self.set_used(*key);

    //     let value = self.cached.get(key);
    //     match value {
    //         Some(v) => Some(v.iter()),
    //         None => None,
    //     }
    // }

    pub fn get(&self, key: &ChunkId) -> Option<&Vec<Chunk>> {
        let value = self.cached.get(key);
        match value {
            Some((vec, _)) => Some(vec),
            None => None,
        }
    }

    pub fn clear_unused(&mut self) -> usize {
        let remove_count;
        {
            let cached: HashSet<_> = self.cached.iter().map(|(k, _)| *k).collect();
            let removed: Vec<_> = cached.difference(&self.used).collect();
            remove_count = removed.len();

            for &k in &removed {
                self.cached.remove(k);
            }
        }

        self.used.clear();
        remove_count
    }
}

#[cfg(test)]
mod cache_tests {
    use voxrs_math::{Aabb, Vec3};

    use super::*;

    #[test]
    fn clear_unused() {
        let mut cache = ChunkCache::new();
        assert_eq!(cache.set_used(1), false);

        let bp_chunk_1 = SafeCloner::new(blueprint::Chunk::new(
            Vec3::ZERO,
            Aabb::unit(),
            Vec::new(),
            Vec::new(),
        ));

        let chunk_1 = Vec::new();
        cache.add(1, bp_chunk_1, chunk_1);

        let bp_chunk_2 = SafeCloner::new(blueprint::Chunk::new(
            Vec3::ZERO,
            Aabb::unit(),
            Vec::new(),
            Vec::new(),
        ));

        let chunk_2 = Vec::new();
        cache.add(2, bp_chunk_2, chunk_2);

        let removed = cache.clear_unused();
        assert_eq!(removed, 0);

        cache.set_used(2);

        let removed = cache.clear_unused();
        assert_eq!(removed, 1);

        assert!(cache.get(&1).is_none());
        assert!(cache.get(&2).is_some());
    }
}
