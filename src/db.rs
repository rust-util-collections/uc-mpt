use crate::errors::MemDBError;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

/// "DB" defines the "trait" of trie and database interaction.
/// You should first write the data to the cache and write the data
/// to the database in bulk after the end of a set of operations.
pub trait DB: Send + Sync {
    type Error: Error;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;

    fn contains(&self, key: &[u8]) -> Result<bool, Self::Error>;

    fn insert(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;

    fn remove(&self, key: &[u8]) -> Result<(), Self::Error>;

    fn insert_batch(&self, kvs: &[(Vec<u8>, Vec<u8>)]) -> Result<(), Self::Error> {
        for (k, v) in kvs.iter() {
            self.insert(k, v)?;
        }
        Ok(())
    }

    fn insert_batch_ref(&self, kvs: &[(&[u8], &[u8])]) -> Result<(), Self::Error> {
        let kvs = kvs
            .iter()
            .map(|(k, v)| (k.to_vec(), v.to_vec()))
            .collect::<Vec<_>>();
        self.insert_batch(&kvs)
    }

    fn remove_batch(&self, keys: &[Vec<u8>]) -> Result<(), Self::Error> {
        for key in keys {
            self.remove(key)?;
        }
        Ok(())
    }

    fn remove_batch_ref(&self, keys: &[&[u8]]) -> Result<(), Self::Error> {
        for key in keys {
            self.remove(key)?;
        }
        Ok(())
    }

    /// If you have a cache,
    /// flush data to the DB from the cache?
    fn flush(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[cfg(test)]
    fn len(&self) -> Result<usize, Self::Error>;

    #[cfg(test)]
    fn is_empty(&self) -> Result<bool, Self::Error>;
}

#[derive(Default, Debug)]
pub struct MemoryDB {
    storage: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl MemoryDB {
    pub fn new() -> Self {
        MemoryDB {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl DB for MemoryDB {
    type Error = MemDBError;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        if let Some(value) = self.storage.read().get(key) {
            Ok(Some(value.clone()))
        } else {
            Ok(None)
        }
    }

    fn insert(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        self.storage.write().insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn contains(&self, key: &[u8]) -> Result<bool, Self::Error> {
        Ok(self.storage.read().contains_key(key))
    }

    fn remove(&self, key: &[u8]) -> Result<(), Self::Error> {
        self.storage.write().remove(key);
        Ok(())
    }

    fn flush(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[cfg(test)]
    fn len(&self) -> Result<usize, Self::Error> {
        Ok(self.storage.try_read().unwrap().len())
    }
    #[cfg(test)]
    fn is_empty(&self) -> Result<bool, Self::Error> {
        Ok(self.storage.try_read().unwrap().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memdb_get() {
        let memdb = MemoryDB::new();
        memdb.insert(b"test-key", b"test-value").unwrap();
        let v = memdb.get(b"test-key").unwrap().unwrap();

        assert_eq!(v, b"test-value")
    }

    #[test]
    fn test_memdb_contains() {
        let memdb = MemoryDB::new();
        memdb.insert(b"test", b"test").unwrap();

        let contains = memdb.contains(b"test").unwrap();
        assert!(contains)
    }

    #[test]
    fn test_memdb_remove() {
        let memdb = MemoryDB::new();
        memdb.insert(b"test", b"test").unwrap();

        memdb.remove(b"test").unwrap();
        let contains = memdb.contains(b"test").unwrap();
        assert!(!contains)
    }
}
