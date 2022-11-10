
pub use serde as dep;

use crate::Flags;

pub struct SetVisitor<T>(std::marker::PhantomData<fn() -> T>);

impl<T> SetVisitor<T> {
    pub fn new() -> Self {
        SetVisitor(std::marker::PhantomData::default())
    }
}

impl<'de, T> dep::de::Visitor<'de> for SetVisitor<T>
where
    T: Flags,
    T::Item: dep::Deserialize<'de>,
{
    type Value = T;

    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "a sequence of set items")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: dep::de::SeqAccess<'de>,
    {
        let mut set = T::EMPTY;
        while let Some(item) = seq.next_element::<T::Item>()? {
            set.insert(item);
        }
        Ok(set)
    }
}