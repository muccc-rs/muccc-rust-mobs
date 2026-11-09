//! This is essentially our own simple reimlementation of indexmap.
//! It's needed for Lua `next(table, key)` builtin.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum Key<T> {
    Sentinel,
    Real(T),
}
use Key::*;
impl<T> Key<T> {
    fn as_real(&self) -> Option<&T> {
        if let Real(real) = self {
            Some(real)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct Item<K, V> {
    value: Option<V>,
    next_key: Key<K>,
    prev_key: Key<K>,
}

#[derive(Debug)]
pub struct LinkedHashMap<K, V> {
    table: HashMap<Key<K>, Item<K, V>>,
}

impl<K, V> Default for LinkedHashMap<K, V>
where
    K: Eq + Hash + Clone + Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> LinkedHashMap<K, V>
where
    K: Eq + Hash + Clone + Debug,
{
    pub fn new() -> Self {
        let item = Item {
            value: None,
            next_key: Sentinel,
            prev_key: Sentinel,
        };
        let mut table = HashMap::new();
        table.insert(Sentinel, item);
        Self { table }
    }

    pub fn insert(&mut self, key: K, value: V) {
        if let Some(v) = self.table.get_mut(&Key::Real(key.clone())) {
            v.value = Some(value);
            return;
        }
        let end = self.table[&Sentinel].prev_key.clone();
        let item = Item {
            value: Some(value),
            next_key: Sentinel,
            prev_key: end.clone(),
        };
        let old = self.table.insert(Key::Real(key.clone()), item);
        assert!(old.is_none());
        self.table.get_mut(&Sentinel).unwrap().prev_key = Key::Real(key.clone());
        self.table.get_mut(&end).unwrap().next_key = Key::Real(key.clone());
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let item = self.table.get(&Key::Real(key.clone()))?;
        Some(item.value.as_ref().unwrap())
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let item = self.table.remove(&Key::Real(key.clone()))?;
        self.table.get_mut(&item.prev_key).unwrap().next_key = item.next_key.clone();
        self.table.get_mut(&item.next_key).unwrap().prev_key = item.prev_key;
        item.value
    }

    pub fn entries(&self) -> impl Iterator<Item = (&K, &V)> {
        let mut key = &self.table.get(&Sentinel).unwrap().next_key;
        std::iter::from_fn(move || {
            let item = self.table.get(&key).unwrap();
            let rv = key.as_real().map(|k| (k, item.value.as_ref().unwrap()));
            key = &item.next_key;
            rv
        })
    }

    pub fn next(&self, key: Option<&K>) -> Option<(&K, &V)> {
        let key = key.map_or(Sentinel, |k| Real(k.clone()));
        let next_key = &self.table.get(&key)?.next_key;
        let item = self.table.get(&next_key)?;
        item.value.as_ref().map(|v| (next_key.as_real().unwrap(), v))
    }

    pub fn len(&self) -> usize {
        self.table.len() - 1
    }
}

//#[cfg(tests)]
mod tests {
    use std::iter;

    use rand::{RngExt, SeedableRng};

    use super::*;

    #[test]
    fn linked_hashmap() {
        let mut m = LinkedHashMap::new();
        m.insert(1, true);
        assert!(m.get(&1).unwrap());
    }

    #[test]
    fn add_and_remove() {
        let mut m = LinkedHashMap::new();
        m.insert(1, "a");
        assert_eq!(m.len(), 1);
        assert_eq!(m.remove(&1), Some("a"));
        assert_eq!(m.len(), 0);
    }

    #[test]
    fn remove_fails_without_element() {
        let mut m: LinkedHashMap<i32, String> = LinkedHashMap::new();
        assert_eq!(m.remove(&1), None);
    }

    #[test]
    fn does_it_iterate() {
        let mut m = LinkedHashMap::new();
        m.insert(1, "a");
        m.insert(2, "b");
        let entries: Vec<_> = m.entries().collect();
        assert_eq!(m.entries().count(), 2);
    }

    #[test]
    fn insert_random_things() {
        use rand::Rng;
        use rand::seq::IndexedRandom;

        let insertable_things = ["a", "b", "c", "d", "e", "dingus", "67", "cucumber"];

        let mut rng = rand::rngs::StdRng::seed_from_u64(67);

        let mut m = LinkedHashMap::new();
        let mut h: HashMap<&str, &str> = HashMap::new();

        for _ in 1..100000 {
            let key = *insertable_things
                .choose(&mut rng)
                .expect("there is always a choice, son");
            if rng.random_ratio(2, 3) {
                let value = *insertable_things
                    .choose(&mut rng)
                    .expect("there is always a choice, son");
                m.insert(key, value);
                h.insert(key, value);
                println!("inserting {key:?}: {value:?}");
            } else {
                let this = m.remove(&key);
                let that = h.remove(key);
                println!("removing {key:?}");
                assert_eq!(this, that);
            }

            assert_eq!(m.len(), h.len());

            let mut elems_ours: Vec<_> = m.entries().collect();
            let mut elems_theirs: Vec<_> = h.iter().collect();

            elems_ours.sort();
            elems_theirs.sort();

            assert_eq!(elems_ours, elems_theirs, "not the same");
        }
    }
}
