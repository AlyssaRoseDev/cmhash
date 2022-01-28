#![cfg_attr(not(test), no_std)]
#![deny(missing_docs, missing_debug_implementations)]
#![feature(bigint_helper_methods, array_chunks)]

//! # cmhash - Core Mersenne Hashing
//!
//! cmhash uses widening multiply and xor to provide fast hashes of machine words
//!
//! Note: This is not a cryptographically secure hashing algorithm and is primarily meant for use in sharding and hash tables

#[cfg(not(loom))]
use core::sync::atomic::AtomicUsize;

#[cfg(loom)]
use loom::sync::atomic::AtomicUsize;

use core::cell::Cell;
use core::sync::atomic::Ordering;

#[cfg(test)]
mod test;

/// Implementations of `Hasher` and `BuildHasher` using fast Mersenne hashing
pub mod hasher;
pub use crate::hasher::*;

// The largest Mersenne Prime that can fit in one word of the target
#[cfg(target_pointer_width = "64")]
const MERSENNE_PRIME: usize = (2 << 61) - 1;

#[cfg(target_pointer_width = "32")]
const MERSENNE_PRIME: usize = (2 << 31) - 1;

#[cfg(target_pointer_width = "16")]
const MERSENNE_PRIME: usize = (2 << 13) - 1;

//Default state is "existential crisis"
#[cfg(target_pointer_width = "64")]
pub(crate) const DEFAULT_STATE: usize = 0xAAAA_AAAA_AAAA_AAAA;

///The default state for the stateful hashers
#[cfg(target_pointer_width = "32")]
pub(crate) const DEFAULT_STATE: usize = 0xAAAA_AAAA;

///The default state for the stateful hashers
#[cfg(target_pointer_width = "16")]
pub(crate) const DEFAULT_STATE: usize = 0xAAAA;

/// A Thread-Local Core Hasher that uses Cell to minimize the cost of shared mutable state

#[derive(Debug)]
pub struct TLCoreHasher(Cell<usize>);

impl TLCoreHasher {
    /// Creates a new [`TLCoreHasher`] with default state.
    pub fn new() -> Self {
        Self::with_state(DEFAULT_STATE)
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
    pub fn hash_word(&self, val: usize) -> usize {
        let state = self.0.get();
        let input = val ^ state;
        let (hash, state) = input.widening_mul(MERSENNE_PRIME);
        self.0.set(state);
        hash
    }

    /// Hashes a slice of bytes by converting to a slice of usize and repeatedly applying [`Self::hash_word`]
    pub fn hash_bytes(&self, bytes: &[u8]) -> usize {
        const N: usize = core::mem::size_of::<usize>();
        let chunks = bytes.array_chunks::<N>();
        let rem = {
            let mut r = chunks.remainder().iter();
            usize::from_ne_bytes([0u8; N].map(|_| *r.next().unwrap_or(&0)))
        };
        chunks
            .map(|c| usize::from_ne_bytes(*c))
            .chain(core::iter::once(rem))
            .fold(0, |val, next| val ^ self.hash_word(next))
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
    pub fn new() -> Self {
        Self::with_state(DEFAULT_STATE)
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
    pub fn hash_word(&self, val: usize) -> usize {
        let state = self.0.load(Ordering::Acquire);
        let input = val ^ state;
        let (hash, state) = input.widening_mul(MERSENNE_PRIME);
        self.0.store(state, Ordering::Release);
        hash
    }

    /// Hashes a slice of bytes by converting to a slice of usize
    /// and repeatedly applying [`Self::hash_word`]
    pub fn hash_bytes(&self, bytes: &[u8]) -> usize {
        const N: usize = core::mem::size_of::<usize>();
        let chunks = bytes.array_chunks::<N>();
        let rem = {
            let mut r = chunks.remainder().iter();
            usize::from_ne_bytes([0u8; N].map(|_| *r.next().unwrap_or(&0)))
        };
        chunks
            .map(|c| usize::from_ne_bytes(*c))
            .chain(core::iter::once(rem))
            .fold(0, |val, next| val ^ self.hash_word(next))
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
pub fn hash_word_stateless(val: usize) -> usize {
    let (hash, state) = (val ^ DEFAULT_STATE).widening_mul(MERSENNE_PRIME);
    hash ^ state
}
