//! Collection utilities for the LedgerDB blockchain.
//!
//! This module provides advanced data structures and utilities for working
//! with collections, including LRU cache, bloom filters, and other specialized containers.

use crate::error::LedgerError;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

/// LRU (Least Recently Used) Cache implementation
#[derive(Debug)]
pub struct LruCache<K, V> {
    capacity: usize,
    map: HashMap<K, (V, usize)>,
    order: VecDeque<K>,
    access_counter: usize,
}

impl<K: Clone + Eq + Hash, V> LruCache<K, V> {
    /// Create a new LRU cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::new(),
            order: VecDeque::new(),
            access_counter: 0,
        }
    }
    
    /// Get value by key
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some((value, _)) = self.map.get_mut(key) {
            self.access_counter += 1;
            *value = (value.clone(), self.access_counter);
            
            // Move to front of order queue
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                let key = self.order.remove(pos).unwrap();
                self.order.push_front(key);
            }
            
            Some(&self.map.get(key).unwrap().0)
        } else {
            None
        }
    }
    
    /// Insert key-value pair
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.access_counter += 1;
        
        if let Some((old_value, _)) = self.map.insert(key.clone(), (value, self.access_counter)) {
            // Key already existed, move to front
            if let Some(pos) = self.order.iter().position(|k| k == &key) {
                let key = self.order.remove(pos).unwrap();
                self.order.push_front(key);
            }
            Some(old_value)
        } else {
            // New key
            self.order.push_front(key.clone());
            
            // Check capacity and evict if necessary
            if self.order.len() > self.capacity {
                if let Some(evicted_key) = self.order.pop_back() {
                    self.map.remove(&evicted_key);
                }
            }
            
            None
        }
    }
    
    /// Remove key-value pair
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some((value, _)) = self.map.remove(key) {
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                self.order.remove(pos);
            }
            Some(value)
        } else {
            None
        }
    }
    
    /// Check if key exists
    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }
    
    /// Get current size
    pub fn len(&self) -> usize {
        self.map.len()
    }
    
    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    
    /// Clear all entries
    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
        self.access_counter = 0;
    }
    
    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// Get all keys in access order (most recent first)
    pub fn keys(&self) -> Vec<&K> {
        self.order.iter().collect()
    }
}

/// Bloom filter for probabilistic membership testing
#[derive(Debug, Clone)]
pub struct BloomFilter {
    bits: Vec<bool>,
    hash_functions: usize,
    size: usize,
}

impl BloomFilter {
    /// Create a new bloom filter
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        let size = Self::optimal_size(expected_items, false_positive_rate);
        let hash_functions = Self::optimal_hash_functions(size, expected_items);
        
        Self {
            bits: vec![false; size],
            hash_functions,
            size,
        }
    }
    
    /// Create bloom filter with specific parameters
    pub fn with_params(size: usize, hash_functions: usize) -> Self {
        Self {
            bits: vec![false; size],
            hash_functions,
            size,
        }
    }
    
    /// Add item to bloom filter
    pub fn add<T: Hash>(&mut self, item: &T) {
        let hashes = self.hash_item(item);
        for hash in hashes {
            let index = (hash as usize) % self.size;
            self.bits[index] = true;
        }
    }
    
    /// Check if item might be in the set
    pub fn contains<T: Hash>(&self, item: &T) -> bool {
        let hashes = self.hash_item(item);
        for hash in hashes {
            let index = (hash as usize) % self.size;
            if !self.bits[index] {
                return false;
            }
        }
        true
    }
    
    /// Clear all bits
    pub fn clear(&mut self) {
        self.bits.fill(false);
    }
    
    /// Get current false positive probability estimate
    pub fn false_positive_probability(&self, items_added: usize) -> f64 {
        let ratio = items_added as f64 / self.size as f64;
        (1.0 - (-self.hash_functions as f64 * ratio).exp()).powi(self.hash_functions as i32)
    }
    
    /// Get filter statistics
    pub fn stats(&self) -> BloomFilterStats {
        let set_bits = self.bits.iter().filter(|&&b| b).count();
        let load_factor = set_bits as f64 / self.size as f64;
        
        BloomFilterStats {
            size: self.size,
            hash_functions: self.hash_functions,
            set_bits,
            load_factor,
        }
    }
    
    /// Calculate optimal size for given parameters
    fn optimal_size(expected_items: usize, false_positive_rate: f64) -> usize {
        let ln2 = std::f64::consts::LN_2;
        let size = -(expected_items as f64 * false_positive_rate.ln()) / (ln2 * ln2);
        size.ceil() as usize
    }
    
    /// Calculate optimal number of hash functions
    fn optimal_hash_functions(size: usize, expected_items: usize) -> usize {
        let ratio = size as f64 / expected_items as f64;
        let hash_functions = ratio * std::f64::consts::LN_2;
        hash_functions.ceil() as usize
    }
    
    /// Generate multiple hash values for an item
    fn hash_item<T: Hash>(&self, item: &T) -> Vec<u64> {
        let mut hashes = Vec::with_capacity(self.hash_functions);
        
        for i in 0..self.hash_functions {
            let mut hasher = DefaultHasher::new();
            item.hash(&mut hasher);
            i.hash(&mut hasher);
            hashes.push(hasher.finish());
        }
        
        hashes
    }
}

/// Bloom filter statistics
#[derive(Debug, Clone)]
pub struct BloomFilterStats {
    pub size: usize,
    pub hash_functions: usize,
    pub set_bits: usize,
    pub load_factor: f64,
}

/// Ring buffer implementation
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    buffer: Vec<Option<T>>,
    capacity: usize,
    head: usize,
    tail: usize,
    size: usize,
}

impl<T> RingBuffer<T> {
    /// Create a new ring buffer with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![None; capacity],
            capacity,
            head: 0,
            tail: 0,
            size: 0,
        }
    }
    
    /// Push item to the back of the buffer
    pub fn push(&mut self, item: T) -> Option<T> {
        let old_item = self.buffer[self.tail].take();
        self.buffer[self.tail] = Some(item);
        
        self.tail = (self.tail + 1) % self.capacity;
        
        if self.size < self.capacity {
            self.size += 1;
            None
        } else {
            self.head = (self.head + 1) % self.capacity;
            old_item
        }
    }
    
    /// Pop item from the front of the buffer
    pub fn pop(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }
        
        let item = self.buffer[self.head].take();
        self.head = (self.head + 1) % self.capacity;
        self.size -= 1;
        
        item
    }
    
    /// Get item at index (0 is front)
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.size {
            return None;
        }
        
        let actual_index = (self.head + index) % self.capacity;
        self.buffer[actual_index].as_ref()
    }
    
    /// Get current size
    pub fn len(&self) -> usize {
        self.size
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
    
    /// Check if buffer is full
    pub fn is_full(&self) -> bool {
        self.size == self.capacity
    }
    
    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// Clear all items
    pub fn clear(&mut self) {
        for item in &mut self.buffer {
            *item = None;
        }
        self.head = 0;
        self.tail = 0;
        self.size = 0;
    }
    
    /// Convert to vector (front to back order)
    pub fn to_vec(&self) -> Vec<T> 
    where 
        T: Clone,
    {
        let mut result = Vec::with_capacity(self.size);
        for i in 0..self.size {
            if let Some(item) = self.get(i) {
                result.push(item.clone());
            }
        }
        result
    }
}

/// Frequency counter for items
#[derive(Debug, Clone)]
pub struct FrequencyCounter<T: Hash + Eq> {
    counts: HashMap<T, usize>,
    total: usize,
}

impl<T: Hash + Eq + Clone> FrequencyCounter<T> {
    /// Create a new frequency counter
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
            total: 0,
        }
    }
    
    /// Add an item
    pub fn add(&mut self, item: T) {
        *self.counts.entry(item).or_insert(0) += 1;
        self.total += 1;
    }
    
    /// Add multiple occurrences of an item
    pub fn add_count(&mut self, item: T, count: usize) {
        *self.counts.entry(item).or_insert(0) += count;
        self.total += count;
    }
    
    /// Get count for an item
    pub fn get_count(&self, item: &T) -> usize {
        self.counts.get(item).copied().unwrap_or(0)
    }
    
    /// Get frequency (0.0 to 1.0) for an item
    pub fn get_frequency(&self, item: &T) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.get_count(item) as f64 / self.total as f64
    }
    
    /// Get most frequent items
    pub fn most_frequent(&self, n: usize) -> Vec<(T, usize)> {
        let mut items: Vec<_> = self.counts.iter().map(|(k, &v)| (k.clone(), v)).collect();
        items.sort_by(|a, b| b.1.cmp(&a.1));
        items.truncate(n);
        items
    }
    
    /// Get least frequent items
    pub fn least_frequent(&self, n: usize) -> Vec<(T, usize)> {
        let mut items: Vec<_> = self.counts.iter().map(|(k, &v)| (k.clone(), v)).collect();
        items.sort_by(|a, b| a.1.cmp(&b.1));
        items.truncate(n);
        items
    }
    
    /// Get total count
    pub fn total(&self) -> usize {
        self.total
    }
    
    /// Get unique items count
    pub fn unique_count(&self) -> usize {
        self.counts.len()
    }
    
    /// Clear all counts
    pub fn clear(&mut self) {
        self.counts.clear();
        self.total = 0;
    }
    
    /// Get all items with their counts
    pub fn items(&self) -> Vec<(T, usize)> {
        self.counts.iter().map(|(k, &v)| (k.clone(), v)).collect()
    }
}

impl<T: Hash + Eq + Clone> Default for FrequencyCounter<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Set operations utilities
pub struct SetUtils;

impl SetUtils {
    /// Union of two hash sets
    pub fn union<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T> {
        a.union(b).cloned().collect()
    }
    
    /// Intersection of two hash sets
    pub fn intersection<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T> {
        a.intersection(b).cloned().collect()
    }
    
    /// Difference of two hash sets (a - b)
    pub fn difference<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T> {
        a.difference(b).cloned().collect()
    }
    
    /// Symmetric difference of two hash sets
    pub fn symmetric_difference<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T> {
        a.symmetric_difference(b).cloned().collect()
    }
    
    /// Check if set a is subset of set b
    pub fn is_subset<T: Hash + Eq>(a: &HashSet<T>, b: &HashSet<T>) -> bool {
        a.is_subset(b)
    }
    
    /// Check if set a is superset of set b
    pub fn is_superset<T: Hash + Eq>(a: &HashSet<T>, b: &HashSet<T>) -> bool {
        a.is_superset(b)
    }
    
    /// Check if two sets are disjoint
    pub fn is_disjoint<T: Hash + Eq>(a: &HashSet<T>, b: &HashSet<T>) -> bool {
        a.is_disjoint(b)
    }
    
    /// Jaccard similarity coefficient
    pub fn jaccard_similarity<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> f64 {
        if a.is_empty() && b.is_empty() {
            return 1.0;
        }
        
        let intersection_size = Self::intersection(a, b).len();
        let union_size = Self::union(a, b).len();
        
        if union_size == 0 {
            0.0
        } else {
            intersection_size as f64 / union_size as f64
        }
    }
}

/// Collection utilities
pub struct CollectionUtils;

impl CollectionUtils {
    /// Partition vector into chunks of specified size
    pub fn chunk<T>(items: Vec<T>, chunk_size: usize) -> Vec<Vec<T>> {
        if chunk_size == 0 {
            return vec![items];
        }
        
        items.chunks(chunk_size).map(|chunk| chunk.to_vec()).collect()
    }
    
    /// Flatten nested vectors
    pub fn flatten<T>(nested: Vec<Vec<T>>) -> Vec<T> {
        nested.into_iter().flatten().collect()
    }
    
    /// Remove duplicates while preserving order
    pub fn dedup_preserve_order<T: Hash + Eq + Clone>(items: Vec<T>) -> Vec<T> {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        
        for item in items {
            if seen.insert(item.clone()) {
                result.push(item);
            }
        }
        
        result
    }
    
    /// Group items by key function
    pub fn group_by<T, K, F>(items: Vec<T>, key_fn: F) -> HashMap<K, Vec<T>>
    where
        K: Hash + Eq,
        F: Fn(&T) -> K,
    {
        let mut groups = HashMap::new();
        
        for item in items {
            let key = key_fn(&item);
            groups.entry(key).or_insert_with(Vec::new).push(item);
        }
        
        groups
    }
    
    /// Find items that appear in all vectors
    pub fn find_common<T: Hash + Eq + Clone>(vectors: &[Vec<T>]) -> Vec<T> {
        if vectors.is_empty() {
            return Vec::new();
        }
        
        let mut common: HashSet<T> = vectors[0].iter().cloned().collect();
        
        for vector in &vectors[1..] {
            let current: HashSet<T> = vector.iter().cloned().collect();
            common = common.intersection(&current).cloned().collect();
        }
        
        common.into_iter().collect()
    }
    
    /// Rotate vector left by n positions
    pub fn rotate_left<T>(mut items: Vec<T>, n: usize) -> Vec<T> {
        if items.is_empty() {
            return items;
        }
        
        let len = items.len();
        let n = n % len;
        items.rotate_left(n);
        items
    }
    
    /// Rotate vector right by n positions
    pub fn rotate_right<T>(mut items: Vec<T>, n: usize) -> Vec<T> {
        if items.is_empty() {
            return items;
        }
        
        let len = items.len();
        let n = n % len;
        items.rotate_right(n);
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lru_cache() {
        let mut cache = LruCache::new(2);
        
        assert_eq!(cache.insert(1, "one"), None);
        assert_eq!(cache.insert(2, "two"), None);
        assert_eq!(cache.get(&1), Some(&"one"));
        
        // This should evict key 2
        assert_eq!(cache.insert(3, "three"), None);
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&3), Some(&"three"));
    }
    
    #[test]
    fn test_bloom_filter() {
        let mut filter = BloomFilter::new(100, 0.01);
        
        filter.add(&"hello");
        filter.add(&"world");
        
        assert!(filter.contains(&"hello"));
        assert!(filter.contains(&"world"));
        assert!(!filter.contains(&"foo")); // Might be false positive, but unlikely
    }
    
    #[test]
    fn test_ring_buffer() {
        let mut buffer = RingBuffer::new(3);
        
        assert_eq!(buffer.push(1), None);
        assert_eq!(buffer.push(2), None);
        assert_eq!(buffer.push(3), None);
        
        // Buffer is full, should return evicted item
        assert_eq!(buffer.push(4), Some(1));
        
        assert_eq!(buffer.get(0), Some(&2));
        assert_eq!(buffer.get(1), Some(&3));
        assert_eq!(buffer.get(2), Some(&4));
    }
    
    #[test]
    fn test_frequency_counter() {
        let mut counter = FrequencyCounter::new();
        
        counter.add("apple");
        counter.add("banana");
        counter.add("apple");
        counter.add("cherry");
        counter.add("apple");
        
        assert_eq!(counter.get_count(&"apple"), 3);
        assert_eq!(counter.get_count(&"banana"), 1);
        assert_eq!(counter.total(), 5);
        
        let most_frequent = counter.most_frequent(2);
        assert_eq!(most_frequent[0].0, "apple");
        assert_eq!(most_frequent[0].1, 3);
    }
    
    #[test]
    fn test_set_operations() {
        let set_a: HashSet<i32> = [1, 2, 3, 4].iter().cloned().collect();
        let set_b: HashSet<i32> = [3, 4, 5, 6].iter().cloned().collect();
        
        let union = SetUtils::union(&set_a, &set_b);
        assert_eq!(union.len(), 6);
        
        let intersection = SetUtils::intersection(&set_a, &set_b);
        assert_eq!(intersection.len(), 2);
        assert!(intersection.contains(&3));
        assert!(intersection.contains(&4));
        
        let similarity = SetUtils::jaccard_similarity(&set_a, &set_b);
        assert!((similarity - 0.333).abs() < 0.01); // 2/6 ≈ 0.333
    }
    
    #[test]
    fn test_collection_utils() {
        let items = vec![1, 2, 3, 4, 5, 6, 7];
        let chunks = CollectionUtils::chunk(items, 3);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], vec![1, 2, 3]);
        assert_eq!(chunks[2], vec![7]);
        
        let with_dups = vec![1, 2, 2, 3, 1, 4, 3];
        let deduped = CollectionUtils::dedup_preserve_order(with_dups);
        assert_eq!(deduped, vec![1, 2, 3, 4]);
    }
}