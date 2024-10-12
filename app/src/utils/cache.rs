// https://users.rust-lang.org/t/solved-what-pattern-would-you-suggest-for-caching-since-theres-no-concept-of-global-heap-variables-in-rust/26086/2
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Clone, Debug)]
pub enum CacheValue {
  StringValue(String),
  HashMapValue(HashMap<String, f64>),
}

#[derive(Clone, Debug)]
pub struct CacheEntry {
  value: CacheValue,
  expiry: Option<Instant>, // Optional expiry time
}

type Data = HashMap<String, CacheEntry>;

lazy_static! {
  static ref CACHE: RwLock<Data> = RwLock::new(HashMap::new());
}

// Function to get a value from the cache
pub fn get_memcache_value(key: &str) -> Option<CacheValue> {
  let cache = CACHE.read().unwrap();

  if let Some(entry) = cache.get(key) {
    // Check for expiry
    if let Some(expiry) = entry.expiry {
      if Instant::now() < expiry {
        return Some(entry.value.clone());
      } else {
        // Remove expired entries
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

pub fn get_memcache_string(key: &str) -> Option<String> {
  if let Some(value) = get_memcache_value(key) {
    if let CacheValue::StringValue(s) = value {
      return Some(s);
    }
  }
  None
}

pub fn get_memcache_hash(key: &str) -> Option<HashMap<String, f64>> {
  if let Some(value) = get_memcache_value(key) {
    if let CacheValue::HashMapValue(h) = value {
      return Some(h);
    }
  }
  None
}
// Function to set a string value in the cache
pub fn set_memcache_string(key: String, value: String, expiry_seconds: Option<u64>) {
  let expiry = expiry_seconds.map(|seconds| Instant::now() + Duration::from_secs(seconds));

  let mut cache = CACHE.write().unwrap();
  cache.insert(
    key,
    CacheEntry {
      value: CacheValue::StringValue(value),
      expiry,
    },
  );
}

// Function to set a HashMap value in the cache
pub fn set_memcache_hashmap(key: String, value: HashMap<String, f64>, expiry_seconds: Option<u64>) {
  let expiry = expiry_seconds.map(|seconds| Instant::now() + Duration::from_secs(seconds));

  let mut cache = CACHE.write().unwrap();
  cache.insert(
    key,
    CacheEntry {
      value: CacheValue::HashMapValue(value),
      expiry,
    },
  );
}
