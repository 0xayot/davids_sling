#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct CacheEntry {
  value: String,
  expiry: Option<Instant>, // Optional expiry time
}

type Data = HashMap<String, CacheEntry>;

lazy_static! {
  static ref CACHE: RwLock<Data> = RwLock::new(HashMap::new());
}

// Function to get a value from the cache
pub fn get_memory_value(key: &str) -> Option<String> {
  let cache = CACHE.read().unwrap();

  if let Some(entry) = cache.get(key) {
    // Check for expiry
    if let Some(expiry) = entry.expiry {
      if Instant::now() < expiry {
        return Some(entry.value.clone());
      } else {
        // Optionally remove expired entries
        drop(cache); // Release read lock before modifying
        let mut cache = CACHE.write().unwrap();
        cache.remove(key);
      }
    } else {
      return Some(entry.value.clone());
    }
  }

  None
}

// Function to set a value in the cache
pub fn set_memory_value(key: String, value: String, expiry_seconds: Option<u64>) {
  let expiry = expiry_seconds.map(|seconds| Instant::now() + Duration::from_secs(seconds));

  let mut cache = CACHE.write().unwrap();
  cache.insert(key, CacheEntry { value, expiry });
}
