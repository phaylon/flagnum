use flagnum::Flags;

/// Enum documentation: An example enum for [`Weekdays`].
#[flagnum::flag(
    /// Set documentation: A set type for [`Weekday`] values.
    #[derive(Default)] pub Weekdays,
    groups(
        /// Group documentation: Contains all [`Weekday`]
        /// values that fall on the weekend.
        pub WEEKEND,
    ),
)]
#[derive(Default)]
pub enum Weekday {
    /// Variant documentation: Monday..
    #[default]
    Monday,
    /// Tuesday..
    Tuesday,
    /// ..
    Wednesday,
    Thursday,
    Friday,
    /// Tagged with `#[groups(WEEKEND)]` and so will appear
    /// in [`Weekdays::WEEKEND`].
    #[groups(WEEKEND)]
    Saturday,
    #[groups(WEEKEND)]
    Sunday,
}

/// The set type can be freely extended with inherent methods and traits.
impl Weekdays {
    pub fn non_weekend_len(self) -> usize {
        self.without(Self::WEEKEND).len()
    }
}
