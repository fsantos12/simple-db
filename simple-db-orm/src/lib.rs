//! # simple-db-orm
//!
//! Object-Relational Mapping layer for the simple-db ecosystem.
//!
//! This crate is a placeholder for a future ORM layer that will provide
//! higher-level abstractions over raw query builders (e.g. model structs,
//! relationships, migrations).
//! No ORM functionality is currently implemented.

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
