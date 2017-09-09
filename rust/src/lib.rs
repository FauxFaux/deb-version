#[macro_use]
extern crate nom;

use std::cmp::Ordering;

mod parse;

pub fn compare_versions(left: &str, right: &str) -> Ordering {
    unimplemented!();
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use compare_versions;

//    #[test]
    fn simple() {
        assert_eq!(Ordering::Less, compare_versions("3.0", "3.1"));
        assert_eq!(Ordering::Greater, compare_versions("3.1", "3.0"));
        assert_eq!(Ordering::Equal, compare_versions("3.0", "3.0"));
    }
}
