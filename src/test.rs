use crate::*;

#[test]
fn nopanic() {
    let val: usize = 0xDEADBEEF;
    let h = CoreHasher::new();
    let hashed1 = h.fast_hash(val);
    let hashed2 = h.fast_hash(val);
    assert_ne!(hashed1, hashed2);
    let h2 = CoreHasher::new();
    let hashed3 = h2.fast_hash(val);
    assert_eq!(hashed1, hashed3);
}

#[test]
fn stateless() {
    let val: usize = 0xF0F0F0F0;
    let hashed1 = stateless_fast_hash(val);
    let hashed2 = stateless_fast_hash(val);
    assert_eq!(hashed1, hashed2)
}

//Mostly to make sure CoreHasher is properly thread-safe, don't know what to assert? 
#[cfg(loom)]
#[test]
fn loomtest() {
    use loom::thread;
    use loom::sync::Arc;
    loom::model(|| {

        let hash1 = Arc::new(CoreHasher::new());
        let hash2 = hash1.clone();

        let t1 = thread::spawn(move || {
            let val: usize = 0xDEADBEEF;
            for _ in 0..3{
                hash1.fast_hash(val);
            }
        });

        let t2 = thread::spawn(move || {
            let val: usize = 0xDEADBEEF;
            for _ in 0..3 {
                hash2.fast_hash(val);
            }
        });

        t1.join().unwrap();
        t2.join().unwrap();
    })
}