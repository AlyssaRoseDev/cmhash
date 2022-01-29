use crate::*;

#[test]
fn nopanic() {
    let val: usize = 0xDEADBEEF;
    let h = CoreHasher::new();
    let hashed1 = h.hash_word(val);
    let hashed2 = h.hash_word(val);
    // Because the hasher state persists, the two hashes cannot be the same
    // even though the value is the same
    assert_ne!(hashed1, hashed2);
    let h2 = CoreHasher::new();
    let hashed3 = h2.hash_word(val);
    // Because we reset the hasher state by constructing a new one, the hashes for this and the
    // first hash of the previous hasher will be equal
    assert_eq!(hashed1, hashed3);
}

#[test]
fn stateless() {
    let val: usize = 0xF0F0F0F0;
    let hashed1 = hash_word_stateless(val);
    let hashed2 = hash_word_stateless(val);
    assert_eq!(hashed1, hashed2)
}

#[test]
fn hasherimpl() {
    use core::hash::Hasher;
    let mut hasher = hasher::CMHasher::new();
    hasher.write(b"Hello, World!");
    let hash1 = hasher.finish();
    hasher.write(b"Hello, World!");
    let hash2 = hasher.finish();
    //Because the hasher state persists, the two hashes cannot be the same
    assert_ne!(hash1, hash2)
}

#[test]
fn statelesshasher() {
    use core::hash::{Hash, Hasher};
    let mut h = hasher::StatelessHasher::new();
    let s = b"Hello, World";
    s.hash(&mut h);
    let hash1 = h.finish();
    s.hash(&mut h);
    let hash2 = h.finish();
    assert_eq!(hash1, hash2);
}

#[test]
fn buildhashers() {
    use core::hash::{BuildHasher, Hash, Hasher};
    let builder = crate::hasher::CMBuildHasher::new();
    let val = b"Lorem ipsum dolor sit amet";
    let hash1 = {
        let mut h = builder.build_hasher();
        val.hash(&mut h);
        h.finish()
    };
    let hash2 = {
        let mut h = builder.build_hasher();
        val.hash(&mut h);
        h.finish()
    };
    assert_eq!(hash1, hash2)
}

//Mostly to make sure CoreHasher is properly thread-safe, don't know what to assert?
#[cfg(loom)]
#[test]
fn loomtest() {
    use loom::sync::Arc;
    use loom::thread;
    loom::model(|| {
        let hash1 = Arc::new(CoreHasher::new());
        let hash2 = hash1.clone();

        let t1 = thread::spawn(move || {
            let val: usize = 0xDEADBEEF;
            for _ in 0..3 {
                hash1.hash_word(val);
            }
        });

        let t2 = thread::spawn(move || {
            let val: usize = 0xDEADBEEF;
            for _ in 0..3 {
                hash2.hash_word(val);
            }
        });

        t1.join().unwrap();
        t2.join().unwrap();
    })
}
