Groupable sets of unit-only enums based on a bitset implementation.

# Description

* Transforms Rust enums and generates corresponding set types, so the set types
  can be extended with associated items and have foreign traits implemented on it.
* Allows grouping of variants into a constant set via the `groups` attributes.
* Provides a set API via the `Flags` trait, allowing generic operations on sets and
  items.
* Automatically chooses the smallest possible represenation given the number of
  variants.
* Allows adding attributes and documentation to the generated set type and group constants.
* Comes with a set oriented serde implementation, available via the `serde` feature.
  The set type serializes and deserializes like a sequence of values belonging to the
  set.
* Auto-implements a number of standard library traits for enums and set types.
* Uses the enum discriminant value to store the set bit information.

# Example

```rust
#[flagnum::flag(
    #[derive(Default)] pub MySet,
    groups(pub GROUP_1, pub GROUP_2),
)]
pub enum MyItem {
    ItemA,
    #[groups(GROUP_1)]
    ItemB,
    #[groups(GROUP_1, GROUP_2)]
    ItemC,
}

use flagnum::Flags;
assert!(! MySet::GROUP_1.contains(MyItem::ItemA));
assert!(MySet::GROUP_1.contains(MyItem::ItemB));
assert!(MySet::GROUP_1.contains(MySet::GROUP_2));
```