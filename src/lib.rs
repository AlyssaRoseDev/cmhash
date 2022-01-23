#![cfg_attr(not(any(std, test)), no_std)]
#![deny(missing_docs)]
#![feature(bigint_helper_methods, array_chunks)]

//! # cmhash - Core Mersenne Hashing
//!
//! cmhash uses widening multiply and xor to provide fast hashes of machine words
//!
//! Note: This is not a cryptographically secure hashing algorithm and is primarily meant for use in sharding and hash tables

#[cfg(loom)]
use loom::sync::atomic::AtomicUsize;

#[cfg(not(loom))]
use core::sync::atomic::AtomicUsize;

use core::cell::Cell;
use core::sync::atomic::Ordering;

#[cfg(test)]
mod test;

// The largest Mersenne Prime that can fit in one word of the target

#[cfg(target_pointer_width = "64")]
const MERSENNE_PRIME: usize = (2 << 61) - 1;

#[cfg(target_pointer_width = "32")]
const MERSENNE_PRIME: usize = (2 << 31) - 1;

#[cfg(target_pointer_width = "16")]
const MERSENNE_PRIME: usize = (2 << 13) - 1;

/// A Thread-Local Core Hasher that uses Cell to minimize the cost of shared mutable state

#[derive(Debug)]
pub struct TLCoreHasher(Cell<usize>);

impl TLCoreHasher {
    /// Creates a new [`TLCoreHasher`] with a default state of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use cmhash::TLCoreHasher;
    ///
    /// assert_eq!(TLCoreHasher::new().get_state(), 0);
    /// ```
    pub fn new() -> Self {
        Self(Cell::new(0))
    }

    /// Creates a new [`TLCoreHasher`] with a specific state.
    ///
    /// # Examples
    ///
    /// ```
    /// use cmhash::TLCoreHasher;
    ///
    /// let state = 0xA5A5A5A5;
    ///
    /// assert_eq!(TLCoreHasher::with_state(state).get_state(), state);
    /// ```
    pub fn with_state(state: usize) -> Self {
        Self(Cell::new(state))
    }

    /// Retrieve the current state.
    pub fn get_state(&self) -> usize {
        self.0.get()
    }

    /// Quickly hash a word sized value.
    ///
    /// # Examples
    ///
    /// ```
    /// use cmhash::TLCoreHasher;
    ///
    /// let tlcore_hasher = TLCoreHasher::new();
    /// assert_eq!(tlcore_hasher.fast_hash(0), 0);
    /// ```
    pub fn fast_hash(&self, val: usize) -> usize {
        let state = self.0.get();
        let input = val ^ state;
        let (hash, state) = input.widening_mul(MERSENNE_PRIME);
        self.0.set(state);
        hash
    }
}

impl Default for TLCoreHasher {
    fn default() -> Self {
        Self::new()
    }
}

///A CoreHasher with support for concurrent access

#[derive(Debug)]
pub struct CoreHasher(AtomicUsize);

impl CoreHasher {
    /// Creates a new [`CoreHasher`] with a default state of 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use cmhash::CoreHasher;
    ///
    /// assert_eq!(CoreHasher::new().get_state(), 0);
    /// ```
    pub fn new() -> Self {
        Self(AtomicUsize::new(0))
    }

    /// Creates a new [`CoreHasher`] with a specific state.
    ///
    /// # Examples
    ///
    /// ```
    /// use cmhash::CoreHasher;
    ///
    /// let state = 0xA5A5A5A5;
    ///
    /// assert_eq!(CoreHasher::with_state(state).get_state(), state);
    /// ```
    pub fn with_state(state: usize) -> Self {
        Self(AtomicUsize::new(state))
    }

    /// Retrieve the current state.
    pub fn get_state(&self) -> usize {
        self.0.load(Ordering::Acquire)
    }

    /// Quickly hash a word sized value.
    ///
    /// # Examples
    ///
    /// ```
    /// use cmhash::CoreHasher;
    ///
    /// let core_hasher = CoreHasher::new();
    /// assert_eq!(core_hasher.fast_hash(0), 0);
    /// ```
    pub fn fast_hash(&self, val: usize) -> usize {
        let state = self.0.load(Ordering::Acquire);
        let input = val ^ state;
        let (hash, state) = input.widening_mul(MERSENNE_PRIME);
        self.0.store(state, Ordering::Release);
        hash
    }
}

impl Default for CoreHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Quickly hash a word sized value without carrying state.
/// Achieves this by calling [`usize::widening_mul`] and xoring the two halves together
///
/// # Examples
///
/// ```
/// use cmhash::stateless_fast_hash;
///
/// assert_eq!(stateless_fast_hash(0), 0);
/// ```
#[inline]
pub fn stateless_fast_hash(val: usize) -> usize {
    let (hash, state) = val.widening_mul(MERSENNE_PRIME);
    hash ^ state
}

#[cfg(hasher)]
/// Implementations of [`Hasher`] and [`BuildHasher`] using fast Mersenne hashing
pub mod hasher {
    extern crate alloc;
    use alloc::vec::Vec;
    use core::cell::Cell;
    use core::hash::{Hasher, BuildHasher};

    ///An implementation of Fast Mersenne Hashing that is compatible with [`Hasher`]
    #[derive(Debug, Default)]
    pub struct CMHasher {
        state: Cell<u64>,
        data: Vec<u8>,
    }

    impl CMHasher {
        /// Creates a new [`CMHasher`].
        pub fn new() -> Self {
            Self {
                state: Cell::new(0),
                data: Vec::new(),
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
            let chunks = self.data.array_chunks::<8>();
            let mut rem = [0u8; 8];
            // TODO: Find a better way to do this?
            (0..8).for_each(|i| rem[i] = *chunks.remainder().get(i).unwrap_or(&0));
            let mut prev = u64::from_ne_bytes(rem);
            for chunk in chunks {
                prev ^= self.hash(u64::from_ne_bytes(*chunk));
            }
            prev
        }

        fn write(&mut self, bytes: &[u8]) {
            self.data.extend_from_slice(bytes);
        }
    }

    /// A [`BuildHasher`] that yields a [`CMHasher`]
    pub struct CMBuildHasher {}

    impl BuildHasher for CMBuildHasher {
        type Hasher = CMHasher;

        fn build_hasher(&self) -> Self::Hasher {
            CMHasher::new()
        }
    }
}
