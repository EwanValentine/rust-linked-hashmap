use std::mem;
use std::borrow::Borrow;
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
    fn bucket<Q>(&self, key: &Q) -> usize
    where
      K: Borrow<Q>,
      Q: Hash + Eq + ?Sized,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() % self.buckets.len() as u64) as usize
    }

    pub fn len(&self) -> usize {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items == 0
    } 

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
      K: Borrow<Q>,
      Q: Hash + Eq + ?Sized, // ?Sized means Q can be str, which isn't sized
    {
        self.buckets[self.bucket(key)]
          .iter()
          .find(|&(ref ekey, _)| ekey.borrow() == key)
          .map(|&(_, ref v)| v)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized, // ?Sized means Q can be str, which isn't sized
    {
        let bucket = self.bucket(key);
        let bucket = &mut self.buckets[bucket];

        // The ? operator with an Option return type, returns a None type immediately if false,
        // whereas with a Result return type, it returns an Err type.
        let i = bucket.iter().position(|&(ref ekey, _)| ekey.borrow() == key)?;

        self.items -= 1;

        // Swap remove, the following case vec![a, b, c, d, e] swap_remove(a, e), would swap,
        // a and e in place, which is more efficient than removing a, then adding the new value
        // onto the end of the vector. Which means you'd end up with vec![e, b, c] etc, which
        // is fine if you do not need your vec to be ordered. Our buckets are not ordered here,
        // so this is fine in this case.
        Some(bucket.swap_remove(i).1)
    }

    // contains_key - checks keys and returns true or false if exists
    pub fn contains_key<Q>(&mut self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized, // ?Sized means Q can be str, which isn't sized
    {
        self.get(key).is_some()
    }
}

pub struct Iter<'a, K, V> {
    map: &'a HashMap<K, V>,
    bucket: usize, // Call store iterators in the buckets themselves? @todo look this up
    at: usize,
    // Could have a yield cound here to prevent 'over yielding'
}

impl <'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {

        // We use a loop here to act as tail call elimination
        // the loop just iterates against a match, which increments
        // the current bucket, and current item position.
        loop {
          match self.map.buckets.get(self.bucket) {
              Some(bucket) => {
                  match bucket.get(self.at) {
                      Some(&(ref k, ref v)) => {
                          self.at += 1;
                          break Some((k, v));
                      }
                      None => {
                          // We've reached the end of the bucket in this case
                          // So we move on to the next bucket, and set the
                          // current position to zero again.
                          self.bucket += 1;
                          self.at = 0;
                          continue;
                      }
                  }
              }

              // No more items
              None => break None,
          };
      }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) ->  Self::IntoIter {
        Iter { map: self, bucket: 0, at: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut map = HashMap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        map.insert("testing", 123);
        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);
        assert_eq!(map.get(&"testing"), Some(&123));
        assert_eq!(map.remove(&"testing"), Some(123));
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
        assert_eq!(map.get(&"testing"), None);
    }

    #[test]
    fn iter() {
        let mut map = HashMap::new();
        map.insert("a", 123);
        map.insert("b", 1231);
        map.insert("c", 1232);
        map.insert("d", 12334);
        map.insert("e", 12345);

        for (&k, &v) in &map {
            match k {
                "a" => assert_eq!(v, 123),
                "b" => assert_eq!(v, 1231),
                "c" => assert_eq!(v, 1232),
                "d" => assert_eq!(v, 12334),
                "e" => assert_eq!(v, 12345),
                _ => unreachable!(),
            }
        }

        assert_eq!((&map).into_iter().count(), 5);
    }
}
