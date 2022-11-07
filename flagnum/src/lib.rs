#![allow(unused_parens)]
#![warn(elided_lifetimes_in_paths)]
#![warn(unused_crate_dependencies)]
#![forbid(unused_must_use)]

pub use flagnum_proc_macro::*;

#[cfg(feature = "serde")]
#[doc(hidden)]
pub use serde;

pub trait Flag: Sized
    + std::fmt::Debug
    + Clone + Copy
    + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash
    + Send + Sync
    + Into<Self::Set>
    + 'static
{
    type Set: Flags;
}

pub trait Flags: Sized
    + std::fmt::Debug
    + Clone + Copy + Default
    + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash
    + Send + Sync
    + From<Self::Item>
    + 'static
{
    type Item: Flag;

    const FULL: Self;
    const EMPTY: Self;
    const ITEMS: &'static [Self::Item];

    #[must_use]
    fn from_item(item: Self::Item) -> Self;

    #[must_use]
    fn from_items(items: &[Self::Item]) -> Self;

    #[must_use]
    fn from_sets(items: &[Self]) -> Self;

    #[must_use]
    fn len(self) -> usize;

    #[must_use]
    fn is_empty(self) -> bool;

    #[must_use]
    fn is_full(self) -> bool;

    #[must_use]
    fn contains<T>(self, other: T) -> bool
    where
        T: Into<Self>;

    #[must_use]
    fn intersection(self, other: Self) -> Self;

    #[must_use]
    fn intersects(self, other: Self) -> bool;

    #[must_use]
    fn with<T>(self, other: T) -> Self
    where
        T: Into<Self>;

    #[must_use]
    fn without<T>(self, other: T) -> Self
    where
        T: Into<Self>;

    #[must_use]
    fn inverse(self) -> Self;

    fn invert(&mut self);

    fn insert<T>(&mut self, other: T)
    where
        T: Into<Self>;

    fn remove<T>(&mut self, other: T)
    where
        T: Into<Self>;

    fn keep<T>(&mut self, other: T)
    where
        T: Into<Self>;

    fn retain<F>(&mut self, is_retained: F)
    where
        F: FnMut(Self::Item) -> bool;
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Iter<T> {
    items: T,
    offset: usize,
}

impl<T> Iter<T> {
    pub fn new(items: T) -> Self {
        Self {
            items,
            offset: 0,
        }
    }
}

impl<T> Iterator for Iter<T>
where
    T: Flags,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = T::ITEMS.get(self.offset).copied() {
            self.offset += 1;
            if self.items.contains(item) {
                return Some(item);
            }
        }
        None
    }
}
