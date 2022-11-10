#![cfg(feature = "serde")]

use flagnum::Flags;

#[test]
fn roundtrip() {
    #[flagnum::flag(Set)]
    enum Item { A, B, C }

    let set = Set::from([Item::A, Item::C]);
    let contents = serde_json::to_string(&set).unwrap();

    let set_rt: Set = serde_json::from_str(&contents).unwrap();
    assert_eq!(set, set_rt);
}

#[test]
fn de() {
    #[flagnum::flag(Set)]
    enum Item { A, B, C }

    assert_eq!(serde_json::from_str::<Set>("[]").unwrap(), Set::EMPTY);
    assert!(serde_json::from_str::<Set>("[0]").is_err());
    assert!(serde_json::from_str::<Set>("{}").is_err());
}