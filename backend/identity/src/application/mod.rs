//! Use cases of the Identity context, its DTOs, its technical ports and its
//! public contract. Depends on the domain only: never on HTTP or SQL.

pub mod contract;
pub mod dto;
pub mod ports;
pub mod use_cases;

#[cfg(test)]
pub(crate) mod test_support;
