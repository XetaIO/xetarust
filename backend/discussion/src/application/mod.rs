//! Use cases of the Discussion context, its DTOs and its outgoing ports.
//! Depends on the domain only: never on HTTP or SQL.

pub mod dto;
pub mod ports;
pub mod use_cases;

#[cfg(test)]
pub(crate) mod test_support;
