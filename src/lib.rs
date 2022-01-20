#![cfg_attr(not(any(test, loom)), no_std)]
#![feature(bigint_helper_methods)]

#[cfg(loom)]
use loom::sync::atomic::AtomicUsize;

#[cfg(not(loom))]
use core::sync::atomic::AtomicUsize;

use core::sync::atomic::Ordering;
use core::cell::Cell;

#[cfg(test)]
mod test;

#[cfg(target_pointer_width = "64")]
const MERSENNE_PRIME: usize = (2 << 61) - 1;

#[cfg(target_pointer_width = "32")]
const MERSENNE_PRIME: usize = (2 << 31) - 1;

#[cfg(target_pointer_width = "16")]
const MERSENNE_PRIME: usize = (2 << 13) - 1;

#[derive(Debug)]
pub struct TLCoreHasher(Cell<usize>);

impl TLCoreHasher {
    pub fn new() -> Self {
        Self(Cell::new(0))
    }

    pub fn with_state(state: usize) -> Self {
        Self(Cell::new(state))
    }

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

#[derive(Debug)]
pub struct CoreHasher(AtomicUsize);

impl CoreHasher {

    pub fn new() -> Self {
        Self(AtomicUsize::new(0))
    }

    pub fn with_state(state: usize) -> Self{
        Self(AtomicUsize::new(state))
    }

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

#[inline]
pub fn stateless_fast_hash(val: usize) -> usize {
    let (hash, state) = val.widening_mul(MERSENNE_PRIME);
    hash ^ state
}