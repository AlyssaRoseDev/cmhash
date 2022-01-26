use core::cell::Cell;
use core::hash::{BuildHasher, Hasher};

///An implementation of Fast Mersenne Hashing that is compatible with [`Hasher`]
#[derive(Debug, Default)]
pub struct CMHasher {
    state: Cell<u64>,
    data: Cell<u64>,
}

impl CMHasher {
    /// Creates a new [`CMHasher`].
    pub fn new() -> Self {
        Self::with_state(0xAAAA_AAAA_AAAA_AAAA)
    }

    /// Creates a new [`CMHasher`] with the specified state
    pub fn with_state(state: u64) -> Self {
        Self {
            state: Cell::new(state),
            data: Cell::new(0),
        }
    }

    fn hash(&self, val: u64) -> u64 {
        let state = self.state.get();
        let input = val ^ state;
        let (hash, state) = input.widening_mul((2 << 61) - 1);
        self.state.set(state);
        hash
    }
}

impl Hasher for CMHasher {
    fn finish(&self) -> u64 {
        self.data.replace(0)
    }

    fn write(&mut self, bytes: &[u8]) {
        let chunks = bytes.array_chunks::<8>();
        let rem = {
            let mut r = chunks.remainder().iter();
            u64::from_ne_bytes([0u8; 8].map(|_| *r.next().unwrap_or(&0)))
        };
        self.data.set(
            chunks
                .map(|c| u64::from_ne_bytes(*c))
                .chain(core::iter::once(rem))
                .fold(0, |val, next| val ^ self.hash(next)),
        );
    }
}

/// A [`BuildHasher`] that yields a [`CMHasher`]
#[derive(Debug)]
pub struct CMBuildHasher {}

impl BuildHasher for CMBuildHasher {
    type Hasher = CMHasher;

    fn build_hasher(&self) -> Self::Hasher {
        CMHasher::new()
    }
}

/// A [`Hasher`] that does not have a persistent internal state for fully deterministic hashing
#[derive(Debug, Default)]
pub struct StatelessHasher {
    data: Cell<u64>,
}

impl StatelessHasher {
    ///Creates a new [`StatelessHasher`]
    pub fn new() -> Self {
        Self { data: Cell::new(0) }
    }

    fn hash(&self, val: u64) -> u64 {
        let (hash, state) = val.widening_mul((2 << 61) - 1);
        hash ^ state
    }
}

impl Hasher for StatelessHasher {
    fn finish(&self) -> u64 {
        self.data.replace(0)
    }

    fn write(&mut self, bytes: &[u8]) {
        let chunks = bytes.array_chunks::<8>();
        let rem = {
            let mut r = chunks.remainder().iter();
            u64::from_ne_bytes([0u8; 8].map(|_| *r.next().unwrap_or(&0)))
        };
        self.data.set(
            chunks
                .map(|c| u64::from_ne_bytes(*c))
                .chain(core::iter::once(rem))
                .fold(0, |val, next| val ^ self.hash(next)),
        );
    }
}

/// A [`BuildHasher`] that yields a [`StatelessHasher`]
#[derive(Debug)]
pub struct StatelessBuildHasher;

impl BuildHasher for StatelessBuildHasher {
    type Hasher = StatelessHasher;

    fn build_hasher(&self) -> Self::Hasher {
        StatelessHasher::new()
    }
}
