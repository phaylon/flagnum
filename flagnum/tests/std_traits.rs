
use flagnum::Flags;

#[test]
fn debug() {
    #[flagnum::flag(DebugTestSet)]
    enum DebugTestItem { DebugTestVariant }

    let item = DebugTestItem::DebugTestVariant;
    let fmt_item = format!("{item:?}");
    assert!(fmt_item.contains("DebugTestVariant"));
    assert!(! fmt_item.contains("DebugTestItem"));

    let set = DebugTestSet::FULL;
    let fmt_set = format!("{set:?}");
    assert!(fmt_set.contains("DebugTestVariant"));
    assert!(! fmt_set.contains("DebugTestItem"));
    assert!(! fmt_set.contains("DebugTestSet"));
}

#[test]
fn from() {
    #[flagnum::flag(Set)]
    enum Item { A, B }

    let set_with_a = Set::from_item(Item::A);
    assert_eq!(Set::from(()), Set::EMPTY);
    assert_eq!(Set::from(Item::A), set_with_a);
    assert_eq!(Set::from(Some(Item::A)), set_with_a);
    assert_eq!(Set::from(None), Set::EMPTY);
    assert_eq!(Set::from([Item::A]), set_with_a);
    assert_eq!(Set::from(&[Item::A]), set_with_a);
    assert_eq!(Set::from(&[Item::A][..]), set_with_a);
    assert_eq!(Set::from(Vec::from([Item::A])), set_with_a);
    assert_eq!(Set::from(&Vec::from([Item::A])), set_with_a);
}

#[test]
fn from_iter() {
    #[flagnum::flag(Set)]
    enum Item { A, B }

    assert_eq!(Set::from_iter([Item::A, Item::B]), Set::FULL);
    assert_eq!(Set::from_iter([Item::A]), Set::from_item(Item::A));
}

#[test]
fn into_iter() {
    #[flagnum::flag(Set)]
    enum Item { A, B }

    assert_eq!(
        Set::FULL.into_iter().collect::<Vec<_>>(),
        Vec::from([Item::A, Item::B]),
    );
    assert_eq!(
        (&Set::FULL).into_iter().collect::<Vec<_>>(),
        Vec::from([Item::A, Item::B]),
    );
}