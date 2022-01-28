use crate::*;

#[test]
fn nopanic() {
    let val: usize = 0xDEADBEEF;
    let h = CoreHasher::new();
    let hashed1 = h.hash_word(val);
    let hashed2 = h.hash_word(val);
    assert_ne!(hashed1, hashed2);
    let h2 = CoreHasher::new();
    let hashed3 = h2.hash_word(val);
    assert_eq!(hashed1, hashed3);
}

#[test]
fn stateless() {
    let val: usize = 0xF0F0F0F0;
    let hashed1 = stateless_fast_hash(val);
    let hashed2 = stateless_fast_hash(val);
    assert_eq!(hashed1, hashed2)
}

#[test]
fn hasherimpl() {
    use core::hash::Hasher;
    let mut hasher = hasher::CMHasher::new();
    hasher.write(b"Hello, World!");
    hasher.finish();
}

#[test]
fn statelesshasher() {
    use core::hash::Hash;
    let mut h = hasher::StatelessHasher::new();
    let s = b"Hello, World";
    let hash1 = s.hash(&mut h);
    let hash2 = s.hash(&mut h);
    assert_eq!(hash1, hash2);
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
