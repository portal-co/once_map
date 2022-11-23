#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "std")]
pub mod sync;

#[cfg(feature = "std")]
pub use sync::{LazyMap, OnceMap};

pub mod unsync;

#[cfg(test)]
mod tests;

use core::{
    borrow::Borrow,
    hash::{BuildHasher, Hash, Hasher},
};

pub trait Equivalent<K: ?Sized> {
    fn equivalent(&self, key: &K) -> bool;
}

impl<Q, K> Equivalent<K> for Q
where
    Q: Eq + ?Sized,
    K: Borrow<Q> + ?Sized,
{
    fn equivalent(&self, key: &K) -> bool {
        self == key.borrow()
    }
}

#[repr(transparent)]
#[derive(Hash)]
struct EquivalentCompat<Q: ?Sized>(Q);

impl<Q: ?Sized> EquivalentCompat<Q> {
    #[inline]
    fn new(q: &Q) -> &Self {
        unsafe { &*(q as *const Q as *const Self) }
    }
}

impl<Q, K> hashbrown::Equivalent<K> for EquivalentCompat<Q>
where
    Q: Equivalent<K> + ?Sized,
{
    #[inline]
    fn equivalent(&self, key: &K) -> bool {
        self.0.equivalent(key)
    }
}

pub trait ToOwnedEquivalent<K>: Equivalent<K> {
    fn to_owned_equivalent(&self) -> K;
}

impl<Q> ToOwnedEquivalent<Q::Owned> for Q
where
    Q: ToOwned + Eq + ?Sized,
{
    fn to_owned_equivalent(&self) -> Q::Owned {
        self.to_owned()
    }
}

fn hash_one<S: BuildHasher, Q: Hash + ?Sized>(hash_builder: &S, key: &Q) -> u64 {
    let mut hasher = hash_builder.build_hasher();
    key.hash(&mut hasher);
    hasher.finish()
}

trait HashMapExt {
    type Key;
    type Value;
    type Hasher;

    fn get_raw_entry<Q>(&self, hash: u64, key: &Q) -> Option<(&Self::Key, &Self::Value)>
    where
        Q: Hash + Equivalent<Self::Key> + ?Sized;

    fn get_raw_entry_mut<Q>(
        &mut self,
        hash: u64,
        key: &Q,
    ) -> hashbrown::hash_map::RawEntryMut<Self::Key, Self::Value, Self::Hasher>
    where
        Q: Hash + Equivalent<Self::Key> + ?Sized;
}

impl<K, V, S> HashMapExt for hashbrown::HashMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    type Key = K;
    type Value = V;
    type Hasher = S;

    fn get_raw_entry<Q>(&self, hash: u64, key: &Q) -> Option<(&Self::Key, &Self::Value)>
    where
        Q: Hash + Equivalent<Self::Key> + ?Sized,
    {
        self.raw_entry()
            .from_key_hashed_nocheck(hash, EquivalentCompat::new(key))
    }

    fn get_raw_entry_mut<Q>(
        &mut self,
        hash: u64,
        key: &Q,
    ) -> hashbrown::hash_map::RawEntryMut<Self::Key, Self::Value, Self::Hasher>
    where
        Q: Hash + Equivalent<Self::Key> + ?Sized,
    {
        self.raw_entry_mut()
            .from_key_hashed_nocheck(hash, EquivalentCompat::new(key))
    }
}

trait InfallibleResult {
    type Ok;

    fn unwrap_infallible(self) -> Self::Ok;
}

impl<T> InfallibleResult for Result<T, core::convert::Infallible> {
    type Ok = T;

    #[inline]
    fn unwrap_infallible(self) -> T {
        match self {
            Ok(v) => v,
            Err(void) => match void {},
        }
    }
}

#[cfg(feature = "ahash")]
use ahash::{AHasher as HasherInner, RandomState as RandomStateInner};

#[cfg(all(not(feature = "ahash"), feature = "std"))]
use std::collections::hash_map::{DefaultHasher as HasherInner, RandomState as RandomStateInner};

#[cfg(all(not(feature = "ahash"), not(feature = "std")))]
compile_error!("Either feature `ahash` or `std` must be enabled");

#[derive(Debug, Clone)]
pub struct RandomState(RandomStateInner);

#[derive(Debug, Clone, Default)]
pub struct DefaultHasher(HasherInner);

impl RandomState {
    #[inline]
    pub fn new() -> Self {
        Self(RandomStateInner::new())
    }
}

impl Default for RandomState {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl core::hash::BuildHasher for RandomState {
    type Hasher = DefaultHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        DefaultHasher(self.0.build_hasher())
    }
}

impl core::hash::Hasher for DefaultHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes)
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.0.write_u8(i)
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.0.write_u16(i)
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.0.write_u32(i)
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0.write_u64(i)
    }

    #[inline]
    fn write_u128(&mut self, i: u128) {
        self.0.write_u128(i)
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.0.write_usize(i)
    }
}
