//! Strongly typed UUID v7 identifiers.

/// Declares a strongly typed UUID v7 identifier so ids of different
/// aggregates can never be mixed up.
///
/// ```
/// xetaravel_kernel::define_id!(
///     /// Identifier of a book.
///     BookId
/// );
/// let id = BookId::generate();
/// assert_eq!(BookId::from(id.as_uuid()), id);
/// ```
#[macro_export]
macro_rules! define_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name($crate::uuid::Uuid);

        impl $name {
            /// Generates a new, time-ordered (UUID v7) identifier.
            pub fn generate() -> Self {
                Self($crate::uuid::Uuid::now_v7())
            }

            /// Wraps an existing UUID (e.g. loaded from storage or a request).
            pub fn from_uuid(uuid: $crate::uuid::Uuid) -> Self {
                Self(uuid)
            }

            /// Returns the underlying UUID.
            pub fn as_uuid(&self) -> $crate::uuid::Uuid {
                self.0
            }
        }

        impl From<$crate::uuid::Uuid> for $name {
            /// Wraps an existing UUID.
            fn from(uuid: $crate::uuid::Uuid) -> Self {
                Self(uuid)
            }
        }

        impl ::std::fmt::Display for $name {
            /// Formats the id as its hyphenated UUID representation.
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    define_id!(
        /// Identifier used by the tests.
        TestId
    );

    #[test]
    fn generated_ids_are_unique() {
        assert_ne!(TestId::generate(), TestId::generate());
    }

    #[test]
    fn round_trips_through_uuid() {
        let uuid = Uuid::now_v7();
        assert_eq!(TestId::from_uuid(uuid).as_uuid(), uuid);
        assert_eq!(TestId::from(uuid).to_string(), uuid.to_string());
    }
}
