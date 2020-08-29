use std::mem;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const INITIAL_NBUCKETS: usize = 1;

pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    items: usize,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            items: 0,
        }
    }
}

// HashMap for keys which have an equality hash check trait
impl<K, V> HashMap<K, V> 
where
    K: Hash + Eq
{
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {

        // If the buckets are empty, or the items are greater than the number of buckets,
        // divided by 4, then resize.
        //
        // Meaning we will always attempt to resize the buckets, if there are more items
        // than a quarter of the amount of buckets. Meaning there will always be four as many 
        // items as buckets.
        //
        // This is kind of arbitrary, but if you had say, a bucket per item, it would use loads
        // of memory. Whereas, if you had one bucket for all items, it would take ages to 
        // traverse all of the items in a bucket.
        if self.buckets.is_empty() || self.items > self.buckets.len() / 4 {
            self.resize(); 
        } 

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let bucket = self.bucket(&key);
        let bucket = &mut self.buckets[bucket];
        
        self.items += 1;
        for &mut (ref ekey, ref mut evalue) in bucket.iter_mut() {
            if ekey == &key {
                return Some(mem::replace(evalue, value));
            }
        }

        bucket.push((key, value));
        None
    }

    // @todo - look-up Amortised costs? 
    // resize - 
    fn resize(&mut self) {

        // Decides how many buckets to create, given the amount of
        // current buckets. It pretty much just doubles them, unless
        // it's 0, then it uses a default value.
        let target_size = match self.buckets.len() {
            0 => INITIAL_NBUCKETS,
            n => 2 * n,
        };

        // Create a new vector of empty buckets with the given target size
        let mut new_buckets = Vec::with_capacity(target_size);

        // Fill the new buckets with empty items to be re-populated
        new_buckets.extend((0..target_size).map(|_| Vec::new()));

        // Drain the old buckets and fill the new ones up again
        for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);

            // @todo - I don't fully understand this, I probaby need to see what
            // hasher returns, to figure out why the modulus of hasher.finish,
            // becomes the new bucket
            let bucket = (hasher.finish() % new_buckets.len() as u64) as usize;
            new_buckets[bucket].push((key, value));
        }

        // In memory replacement of the old and new buckets list
        mem::replace(&mut self.buckets, new_buckets);
    }

    // bucket is a convenience method for figuring out the 
    // bucket for a given key
    fn bucket(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() % self.buckets.len() as u64) as usize
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let bucket = self.bucket(key);
        self.buckets[bucket]
          .iter()
          .find(|&(ref ekey, _)| ekey == key)
          .map(|&(_, ref v)| v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut map = HashMap::new();
        map.insert("testing", 123);
        assert_eq!(map.get(&"testing"), Some(&123));
    }
}
