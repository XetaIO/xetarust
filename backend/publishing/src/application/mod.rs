//! Use cases of the Publishing context, its DTOs, its outgoing ports and its
//! public contract. Depends on the domain only: never on HTTP or SQL.

pub mod contract;
pub mod dto;
pub mod ports;
pub mod use_cases;
pub mod views;

#[cfg(test)]
pub(crate) mod test_support;
