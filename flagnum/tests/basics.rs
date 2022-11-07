
use flagnum::{Flags};

#[flagnum::flag(Weekdays)]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    #[groups(WEEKEND)]
    Sat,
    #[groups(WEEKEND, CLOSED)]
    Sun,
}

#[test]
fn len() {
    assert_eq!(Weekdays::FULL.len(), 7);
    assert_eq!(Weekdays::EMPTY.len(), 0);
    assert_eq!(Weekdays::WEEKEND.len(), 2);
    assert_eq!(Weekdays::CLOSED.len(), 1);
}

#[test]
fn is_empty() {
    assert!(Weekdays::EMPTY.is_empty());
    assert!(! Weekdays::CLOSED.is_empty());
    assert!(! Weekdays::FULL.is_empty());
}

#[test]
fn is_full() {
    assert!(Weekdays::FULL.is_full());
    assert!(! Weekdays::EMPTY.is_full());
    assert!(! Weekdays::CLOSED.is_full());
}

#[test]
fn contains() {
    assert!(Weekdays::WEEKEND.contains(Weekday::Sat));
    assert!(! Weekdays::WEEKEND.contains(Weekday::Fri));

    assert!(Weekdays::WEEKEND.contains(Weekdays::CLOSED));
    assert!(! Weekdays::CLOSED.contains(Weekdays::WEEKEND));
}

#[test]
fn intersections() {
    let fri_sat = Weekdays::from_iter([Weekday::Fri, Weekday::Sat]);

    assert!(Weekdays::WEEKEND.intersects(fri_sat));
    assert!(! Weekdays::CLOSED.intersects(fri_sat));

    assert_eq!(Weekdays::WEEKEND.intersection(fri_sat), Weekday::Sat.into());
    assert!(Weekdays::CLOSED.intersection(fri_sat).is_empty());
}

#[test]
fn with() {
    assert_eq!(
        Weekdays::WEEKEND.with(Weekday::Mon),
        Weekdays::from_iter([Weekday::Sat, Weekday::Sun, Weekday::Mon]),
    );
    assert_eq!(
        Weekdays::WEEKEND.with(Weekdays::from_iter([Weekday::Mon, Weekday::Tue])),
        Weekdays::from_iter([Weekday::Sat, Weekday::Sun, Weekday::Mon, Weekday::Tue]),
    );
}

#[test]
fn without() {
    assert_eq!(
        Weekdays::WEEKEND.without(Weekday::Sat),
        Weekday::Sun.into(),
    );
    assert_eq!(
        Weekdays::FULL.without(Weekdays::WEEKEND),
        Weekdays::from_iter([Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri]),
    );
}

#[test]
fn inverse() {
    assert_eq!(
        Weekdays::WEEKEND.inverse(),
        Weekdays::from_iter([Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri]),
    );
    let mut days = Weekdays::WEEKEND;
    days.invert();
    assert_eq!(
        days,
        Weekdays::from_iter([Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri]),
    );
}