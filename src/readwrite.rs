use std::{collections::HashMap, ops::{Deref, DerefMut}, sync::Arc, hash::Hash};

/// read or read-write mode smart pointer
pub enum ReadWrite<T>
where
    T: Clone,
{
    R(Arc<T>),
    Rw(Arc<T>),
}

impl<T> ReadWrite<T> 
where 
    T: Clone 
{
    pub fn new(t : T) -> Self {
        ReadWrite::Rw(Arc::new(t))
    }

    pub fn clone_read(&self) -> Self {
        match self {
            ReadWrite::R(arc) => ReadWrite::R(Arc::clone(arc)),
            ReadWrite::Rw(arc) => {
                ReadWrite::R(Arc::clone(arc))
            }
        }
    }
    
    pub fn strong_count(&self) -> usize {
        match self {
            ReadWrite::R(arc) => Arc::strong_count(arc),
            ReadWrite::Rw(arc) => Arc::strong_count(arc),
        }
    }
}

impl<T> Deref for ReadWrite<T>
where 
    T: Clone,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            ReadWrite::R(arc) => arc.deref(),
            ReadWrite::Rw(arc) => arc.deref(),
        }
    }
}

/// only ReadWrite::Rw can deref mut
impl<T> DerefMut for ReadWrite<T>
where 
    T: Clone,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            ReadWrite::R(_) => panic!("ReadWrite::R can not modified"),
            ReadWrite::Rw(arc) => {
                // not clonned before, just use self
                if Arc::strong_count(arc) == 1 {
                    Arc::get_mut(arc).unwrap()
                } else { // if already clonned, clone T and write to it
                    let clonned = <T as Clone>::clone(arc);
                    *arc = Arc::new(clonned);
                    Arc::get_mut(arc).unwrap()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;

    #[derive(Clone)]
    struct MyStruct {
        val: i32,
    }

    #[test]
    fn test_new() {
        let my = MyStruct { val: 10 };
        let rw = ReadWrite::new(my);
        assert_eq!(rw.val, 10);
    }

    #[test]
    fn write_test() {
        let my = MyStruct { val: 10 };
        let mut rw = ReadWrite::new(my);

        rw.val = 20;
        assert_eq!(rw.val, 20);
    }

    #[test]
    fn test_clone_read_share_internally() {
        let rw = ReadWrite::new(MyStruct { val: 10 });
        assert_eq!(rw.strong_count(), 1);

        let cloned = rw.clone_read();
        assert_eq!(rw.strong_count(), 2);
        assert_eq!(cloned.strong_count(), 2);
    }

    #[test]
    fn write_after_clone_clone_original_value() {
        let mut rw = ReadWrite::new(MyStruct { val: 10 });
        let clonned = rw.clone_read();
        
        rw.val = 100;

        assert_eq!(rw.strong_count(), 1);
        assert_eq!(clonned.strong_count(), 1);

        assert_eq!(rw.val, 100);
        assert_eq!(clonned.val, 10);
    }

    #[test]
    fn can_pass_between_threads() {
        let rw = ReadWrite::new(MyStruct { val: 10 });
        let clonned = rw.clone_read();

        let handle = thread::spawn(move || {
            assert_eq!(clonned.val, 10);
        });

        handle.join().unwrap();
    }
}

pub struct Cache<K, V>
where
    K: Eq + Hash + Copy,
    V: Clone,
{
    hashmap: HashMap<K, ReadWrite<V>>,
}

impl<K, V> Cache<K, V> 
where 
    K: Eq + Hash + Copy,
    V: Clone,
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
    pub fn add(&mut self, key: K, val: ReadWrite<V>) -> bool {
        let prev = self.hashmap.insert(key, val);
        matches!(prev, Some(_))
    } 

    pub fn get(&self, key: &K) -> Option<&V> {
        let value = self.hashmap.get(key);
        match value {
            Some(v) => Some(v.deref()),
            None => None,
        }
    }

    pub fn len(&self) -> usize {
        self.hashmap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hashmap.is_empty()
    }

    pub fn clear_unused(&mut self) -> usize {
        let mut removes = Vec::new();

        for (k, v) in &self.hashmap {
            if v.strong_count() == 1 {
                removes.push(*k);
            }
        }

        for k in &removes {
            self.hashmap.remove(k);
        }

        removes.len()
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;

    #[test]
    fn add_cache() {
        let mut cache = Cache::new();
        let exists = cache.add(1, ReadWrite::new("1".to_string()));
        assert_eq!(exists, false);

        let exists = cache.add(1, ReadWrite::new("1".to_string()));
        assert_eq!(exists, true);

        let exists = cache.add(2, ReadWrite::new("2".to_string()));
        assert_eq!(exists, false);
    }

    #[test]
    fn clear_unused() {
        let mut cache = Cache::new();

        let mut rw = ReadWrite::new("1".to_string());
        let r = rw.clone_read();

        cache.add(1, r);

        let cleared_count = cache.clear_unused();
        assert_eq!(cleared_count, 0);
        assert_eq!(cache.len(), 1);

        *rw = "2".to_string();
        
        let cleared_count = cache.clear_unused();
        assert_eq!(cleared_count, 1);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_get() {
        let mut cache = Cache::new();
        cache.add(1, ReadWrite::new("1".to_string()));

        let cached = cache.get(&1);
        assert_eq!(cached, Some(&"1".to_string()));
    }
}