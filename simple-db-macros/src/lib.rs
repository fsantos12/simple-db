//! # simple-db-macros
//!
//! Procedural macros for the simple-db ecosystem.
//!
//! This crate is a placeholder for future derive macros (e.g. `#[derive(DbModel)]`).
//! No macros are currently implemented.

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
