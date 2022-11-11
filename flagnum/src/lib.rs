#![allow(unused_parens)]
#![warn(elided_lifetimes_in_paths)]
#![forbid(unused_must_use)]

#![doc = include_str!("../../README.md")]
//!
//! See the [`example`](crate::example) module for a detailed usage and generated API example.
//!
//! See the [`Flags`] trait for examples of the trait API.
//!
//! # Standard Library Traits
//!
//! These traits are automatically implemented for both enum and set types:
//!
//! * [`Debug`](std::fmt::Debug) with a sequence based implementation for sets
//! * [`Clone`] and [`Copy`]
//! * [`PartialEq`] and [`Eq`]
//! * [`PartialOrd`] and [`Ord`]
//! * [`Hash`](std::hash::Hash)
//!
//! These traits are also automatically implemented for set types:
//!
//! * [`From`] for various types
//! * [`FromIterator`] for anything that can be turned into a set
//! * [`IntoIterator`]
//! * [`Extend`] for iterators over anything that can be turned into a set

/// Entry point for enum and set type code generation.
///
/// See the general [flagnum] documentation for general usage information.
///
/// See the [`example`](crate::example) module for an example of using this attribute.
pub use flagnum_proc_macro::flag;

extern crate self as flagnum;


#[cfg(feature = "serde")]
#[doc(hidden)]
pub mod feature_serde;

#[cfg(any(doctest, doc, test))]
pub mod example;

/// A trait implemented for all transformed enum types.
///
/// All set functionality is available on the generated set types or via the
/// [`Flags`](Flags) trait.
///
/// Automatically includes bounds for common automatically implemented traits as well
/// as
///
/// * The [`Send`] and [`Sync`] traits.
/// * An `Into` bound for converting the item type into a corresponding set.
/// * A `'static` lifetime bound.
pub trait Flag: Sized
    + std::fmt::Debug
    + Clone + Copy
    + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash
    + Send + Sync
    + Into<Self::Set>
    + 'static
{
    /// The set type associated with this kind of item.
    type Set: Flags;
}

/// A trait implemented by all generated set types.
///
/// Some associated functions are also generated as inherent `const` variants.
/// The constructors do have generic counterparts, but they are still provided as
/// part of the trait interface for symmetry.
///
/// You can see the [`Weekdays`](crate::example::Weekdays) example for a list of
/// generated inherent members.
///
/// Automatically includes bounds for common automatically implemented traits as well
/// as
///
/// * The [`Send`] and [`Sync`] traits.
/// * A `From` bound for converting from the corresponding item type.
/// * A `'static` lifetime bound.
pub trait Flags: Sized
    + std::fmt::Debug
    + Clone + Copy
    + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash
    + Send + Sync
    + From<Self::Item>
    + 'static
{
    /// The type of items contain in this set.
    ///
    /// This is the type of the enum the set type was generated for.
    type Item: Flag;

    /// A predefined empty set.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// assert!(Colors::EMPTY.is_empty());
    /// assert_eq!(Colors::EMPTY.len(), 0);
    /// ```
    const EMPTY: Self;

    /// A predefined set containing all items.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// assert!(Colors::FULL.is_full());
    /// assert_eq!(Colors::FULL.len(), 3);
    /// assert!(Colors::FULL.contains(Color::Red));
    /// assert!(Colors::FULL.contains(Color::Green));
    /// assert!(Colors::FULL.contains(Color::Blue));
    /// ```
    const FULL: Self;

    /// A static slice of all available items.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// assert_eq!(
    ///     Colors::ITEMS,
    ///     &[Color::Red, Color::Green, Color::Blue],
    /// );
    /// ```
    const ITEMS: &'static [Self::Item];

    /// Construct a set from a single item.
    ///
    /// Also available as an inherent `const` variant on the generated set types
    /// ([Example](crate::example::Weekdays::from_item)).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let colors = Colors::from_item(Color::Green);
    /// assert!(colors.contains(Color::Green));
    /// assert!(! colors.contains(Color::Red));
    /// ```
    #[must_use]
    fn from_item(item: Self::Item) -> Self;

    /// Construct a set from a slice of items.
    ///
    /// Also available as an inherent `const` variant on the generated set types
    /// ([Example](crate::example::Weekdays::from_items)).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let colors = Colors::from_items(&[Color::Green]);
    /// assert!(colors.contains(Color::Green));
    /// assert!(! colors.contains(Color::Red));
    /// ```
    #[must_use]
    fn from_items(items: &[Self::Item]) -> Self;

    /// Construct a set from a slice of sets.
    ///
    /// Also available as an inherent `const` variant on the generated set types
    /// ([Example](crate::example::Weekdays::from_sets)).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let rg = Colors::from_items(&[Color::Red, Color::Green]);
    /// let gb = Colors::from_items(&[Color::Green, Color::Blue]);
    /// let colors = Colors::from_sets(&[rg, gb]);
    /// assert!(colors.is_full());
    /// ```
    #[must_use]
    fn from_sets(items: &[Self]) -> Self;

    /// The number of items in the set.
    ///
    /// Also available as an inherent `const` variant on the generated set types
    /// ([Example](crate::example::Weekdays::len)).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let colors = Colors::from_item(Color::Green);
    /// assert_eq!(colors.len(), 1);
    /// assert_eq!(Colors::EMPTY.len(), 0);
    /// assert_eq!(Colors::FULL.len(), 3);
    /// ```
    #[must_use]
    fn len(self) -> usize;

    /// Predicate determining if the set is empty.
    ///
    /// Also available as an inherent `const` variant on the generated set types
    /// ([Example](crate::example::Weekdays::is_empty)).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let colors = Colors::from_item(Color::Green);
    /// assert!(! colors.is_empty());
    /// assert!(! Colors::FULL.is_empty());
    /// assert!(Colors::EMPTY.is_empty());
    /// ```
    #[must_use]
    fn is_empty(self) -> bool;

    /// Predicate determining if the set contains all items.
    ///
    /// Also available as an inherent `const` variant on the generated set types
    /// ([Example](crate::example::Weekdays::is_full)).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let colors = Colors::from_item(Color::Green);
    /// assert!(! colors.is_full());
    /// assert!(! Colors::EMPTY.is_full());
    /// assert!(Colors::FULL.is_full());
    /// ```
    #[must_use]
    fn is_full(self) -> bool;

    /// Predicate to check if one set contains another.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let colors = Colors::from_item(Color::Green);
    /// assert!(Colors::FULL.contains(colors));
    /// assert!(! Colors::EMPTY.contains(colors));
    /// assert!(colors.contains(Color::Green));
    /// assert!(! colors.contains(Color::Red));
    /// ```
    #[must_use]
    fn contains<T>(self, other: T) -> bool
    where
        T: Into<Self>;

    /// Predicate to check if two sets have any overlap.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let rg = Colors::from_iter([Color::Red, Color::Green]);
    /// let gb = Colors::from_iter([Color::Green, Color::Blue]);
    ///
    /// assert!(rg.has_overlap(gb));
    /// assert!(! rg.has_overlap(Color::Blue));
    /// ```
    #[must_use]
    fn has_overlap<T>(self, other: T) -> bool
    where
        T: Into<Self>;

    /// A set containing items common to both sets.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let rg = Colors::from_iter([Color::Red, Color::Green]);
    /// let gb = Colors::from_iter([Color::Green, Color::Blue]);
    ///
    /// assert_eq!(rg.overlap(gb), Color::Green.into());
    /// ```
    #[must_use]
    fn overlap<T>(self, other: T) -> Self
    where
        T: Into<Self>;

    /// Combine two sets.
    ///
    /// This is the functional version of [`Flags::insert`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let rg = Colors::from_iter([Color::Red, Color::Green]);
    /// let gb = Colors::from_iter([Color::Green, Color::Blue]);
    ///
    /// assert!(rg.with(gb).is_full());
    /// assert!(rg.with(Color::Blue).is_full());
    /// ```
    #[must_use]
    fn with<T>(self, other: T) -> Self
    where
        T: Into<Self>;

    /// A set with all items from the current set without those found in another.
    ///
    /// This is the functional version of [`Flags::remove`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let rg = Colors::from_iter([Color::Red, Color::Green]);
    /// let gb = Colors::from_iter([Color::Green, Color::Blue]);
    ///
    /// assert_eq!(rg.without(gb), Color::Red.into());
    /// assert_eq!(Colors::FULL.without(Color::Blue), rg);
    /// ```
    #[must_use]
    fn without<T>(self, other: T) -> Self
    where
        T: Into<Self>;

    /// A set containing all items not in the current set.
    ///
    /// This is the functional version of [`Flags::invert`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let rg = Colors::from_iter([Color::Red, Color::Green]);
    ///
    /// assert_eq!(rg.missing(), Color::Blue.into());
    /// assert_eq!(Colors::FULL.missing(), Colors::EMPTY);
    /// ```
    #[must_use]
    fn missing(self) -> Self;

    /// Modify the set to only contain items it currently does not
    /// contain.
    ///
    /// This is the self-modifying version of [`Flags::missing`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let mut colors = Colors::from_iter([Color::Red, Color::Green]);
    /// colors.invert();
    /// assert_eq!(colors, Color::Blue.into());
    /// ```
    fn invert(&mut self);

    /// Add items to the set.
    ///
    /// This is the self-modifying version of [`Flags::with`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let mut colors = Colors::from_iter([Color::Red, Color::Green]);
    /// colors.insert(Color::Blue);
    /// assert!(colors.is_full());
    /// ```
    fn insert<T>(&mut self, other: T)
    where
        T: Into<Self>;

    /// Remove items from the set.
    ///
    /// This is the self-modifying version of [`Flags::without`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let mut colors = Colors::FULL;
    /// colors.remove(Color::Blue);
    /// assert_eq!(colors, Colors::from_iter([Color::Red, Color::Green]));
    /// ```
    fn remove<T>(&mut self, other: T)
    where
        T: Into<Self>;

    /// Keep only items that are in the given set.
    ///
    /// This is the set value based version of [`Flags::retain`] and
    /// [`Flags::retained`].
    /// It is also a self-modifying version of [`Flags::overlap`].
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let mut colors = Colors::from_iter([Color::Red, Color::Green]);
    /// colors.keep(Colors::from_iter([Color::Green, Color::Blue]));
    /// assert_eq!(colors, Color::Green.into());
    /// ```
    fn keep<T>(&mut self, other: T)
    where
        T: Into<Self>;

    /// Keep only those items that are accepted by the predicate.
    ///
    /// This is the self-modifying version of [`Flags::retained`].
    /// See [`Flags::overlap`] for a set value based operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let mut colors = Colors::from_iter([Color::Red, Color::Green]);
    /// colors.retain(|color| color == Color::Green);
    /// assert_eq!(colors, Color::Green.into());
    /// ```
    fn retain<F>(&mut self, is_retained: F)
    where
        F: FnMut(Self::Item) -> bool;

    /// A new set containing those items of the current set that are
    /// accepted by the predicate.
    ///
    /// This is the functional version of [`Flags::retain`].
    /// See [`Flags::keep`] for a set value based operation.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::Flags;
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let mut colors = Colors::from_iter([Color::Red, Color::Green]);
    /// let greens = colors.retained(|color| color == Color::Green);
    /// assert_eq!(greens, Color::Green.into());
    /// ```
    fn retained<F>(self, is_retained: F) -> Self
    where
        F: FnMut(Self::Item) -> bool;
}

/// An iterator over the items in a [flagnum] set.
///
/// # Example
///
/// ```rust
/// # use flagnum::{Flags, Iter};
/// #[flagnum::flag(Colors)]
/// enum Color { Red, Green, Blue }
///
/// let colors: Vec<Color> = Colors::FULL.into_iter().collect();
/// assert_eq!(colors, vec![Color::Red, Color::Green, Color::Blue]);
///
/// impl Default for Colors {
///     fn default() -> Self { Self::FULL }
/// }
///
/// // The default iterator will iterate over the default set.
/// let iter: Iter<Colors> = Iter::default();
/// let colors: Vec<Color> = iter.collect();
/// assert_eq!(colors, vec![Color::Red, Color::Green, Color::Blue]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Iter<T> {
    items: T,
    offset: usize,
}

impl<T> Iter<T> {

    /// Construct an iterator over all items in the given set.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use flagnum::{Flags, Iter};
    /// #[flagnum::flag(Colors)]
    /// enum Color { Red, Green, Blue }
    ///
    /// let no_green = Colors::FULL.without(Color::Green);
    /// let colors: Vec<Color> = Iter::new(no_green).collect();
    /// assert_eq!(colors, vec![Color::Red, Color::Blue]);
    /// ```
    pub fn new(set: T) -> Self {
        Self {
            items: set,
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
