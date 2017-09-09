//#[macro_use]
//extern crate nom;
//mod parse;

use std::cmp::Ordering;

fn epoch(version: &str) -> (&str, &str) {
    if let Some(colon) = version.find(':') {
        (&version[..colon], &version[colon + 1..])
    } else {
        ("", version)
    }
}

fn debian_revision(version: &str) -> (&str, &str) {
    if let Some(hyphen) = version.rfind('-') {
        (&version[..hyphen], &version[hyphen + 1..])
    } else {
        (version, "")
    }
}

fn split_point<F>(from: &str, pat: F) -> usize
where
    F: Fn(char) -> bool,
{
    for (pos, chr) in from.chars().enumerate() {
        if !pat(chr) {
            return pos;
        }
    }
    from.len()
}

fn take_while<F>(from: &str, pat: F) -> (&str, &str)
where
    F: Fn(char) -> bool,
{
    let point = split_point(from, pat);
    (&from[..point], &from[point..])
}


fn take_component(version: &str) -> ((&str, &str), &str) {
    let (alpha_part, version) = take_while(version, |chr: char| !chr.is_digit(10));
    let (num_part, version) = take_while(version, |chr: char| chr.is_digit(10));

    ((alpha_part, num_part), version)
}

pub fn compare_digits(left: &str, right: &str) -> Ordering {
    if left == right {
        return Ordering::Equal;
    }

    let left = left.trim_left_matches('0');
    let right = right.trim_left_matches('0');

    match left.len().cmp(&right.len()) {
        Ordering::Equal => left.cmp(&right),
        other => other,
    }
}

pub fn is_ascii_letter(chr: char) -> bool {
    (chr >= 'a' && chr <= 'z') || (chr >= 'A' && chr <= 'Z')
}

pub fn compare_non_digit(left: char, right: char) -> Ordering {
    if left == right {
        return Ordering::Equal;
    }

    if '~' == left {
        return Ordering::Less;
    }

    if '~' == right {
        return Ordering::Greater;
    }

    let left_letter = is_ascii_letter(left);
    let right_letter = is_ascii_letter(right);

    if left_letter == right_letter {
        return left.cmp(&right);
    }

    if left_letter {
        return Ordering::Less;
    }

    return Ordering::Greater;
}

pub fn compare_non_digits(left: &str, right: &str) -> Ordering {
    if left == right {
        return Ordering::Equal;
    }

    let mut left = left.chars();
    let mut right = right.chars();

    loop {
        if let Some(l) = left.next() {
            if let Some(r) = right.next() {
                match compare_non_digit(l, r) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            } else {
                return if '~' == l {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            }
        } else {
            return if let Some(r) = right.next() {
                if '~' == r {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            } else {
                // implies strings are the same, which we test for
                unreachable!();
            };
        }
    }
}

pub fn compare_simple_version(mut left: &str, mut right: &str) -> Ordering {
    if left == right {
        return Ordering::Equal;
    }

    loop {
        let ((left_alpha, left_digit), left_remainder) = take_component(left);
        let ((right_alpha, right_digit), right_remainder) = take_component(right);

        match compare_non_digits(left_alpha, right_alpha) {
            Ordering::Equal => {}
            other => return other,
        }


        match compare_digits(left_digit, right_digit) {
            Ordering::Equal => {}
            other => return other,
        }

        if left_remainder.is_empty() && right_remainder.is_empty() {
            return Ordering::Equal;
        }

        left = left_remainder;
        right = right_remainder;
    }
}

pub fn compare_versions(left: &str, right: &str) -> Ordering {
    if left == right {
        return Ordering::Equal;
    }

    let (left_epoch, left) = epoch(left);
    let (right_epoch, right) = epoch(right);

    match compare_digits(left_epoch, right_epoch) {
        Ordering::Equal => {}
        other => return other,
    }

    let (left, left_debian) = debian_revision(left);
    let (right, right_debian) = debian_revision(right);

    match compare_simple_version(left, right) {
        Ordering::Equal => {}
        other => return other,
    }

    compare_simple_version(left_debian, right_debian)

}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use compare_versions;

    #[test]
    fn test_epoch() {
        use epoch;

        assert_eq!(("", "17"), epoch("17"));
        assert_eq!(("1", "17"), epoch("1:17"));

        // not valid, but kind of implied by the spec?
        assert_eq!(("1", "17:19"), epoch("1:17:19"));
    }

    #[test]
    fn test_extract_revision() {
        use debian_revision;

        assert_eq!(("17", ""), debian_revision("17"));
        assert_eq!(("17", "2"), debian_revision("17-2"));
        assert_eq!(("17-2", "3"), debian_revision("17-2-3"));
    }

    #[test]
    fn test_take_component() {
        use take_component;

        assert_eq!((("abc", "123"), "def456"), take_component("abc123def456"));
        assert_eq!((("a", "123"), "def456"), take_component("a123def456"));
        assert_eq!((("abc", "1"), "def456"), take_component("abc1def456"));
        assert_eq!((("a", "1"), "def456"), take_component("a1def456"));

        assert_eq!((("", "12"), "bc34"), take_component("12bc34"));
        assert_eq!((("", "12"), "b34"), take_component("12b34"));
        assert_eq!((("", "1"), "bc34"), take_component("1bc34"));
        assert_eq!((("", "1"), "b34"), take_component("1b34"));

        assert_eq!((("", "17"), ""), take_component("17"));
        assert_eq!((("", "1"), ""), take_component("1"));
        assert_eq!((("ab", ""), ""), take_component("ab"));
        assert_eq!((("a", ""), ""), take_component("a"));

        assert_eq!((("", ""), ""), take_component(""));
    }

    #[test]
    fn test_compare_digits() {
        use std::cmp::Ordering::*;
        use compare_digits;

        assert_eq!(Equal, compare_digits("1", "1"));
        assert_eq!(Equal, compare_digits("1", "01"));
        assert_eq!(Equal, compare_digits("100", "0100"));
        assert_eq!(Equal, compare_digits("0100", "100"));
        assert_eq!(Equal, compare_digits("00000100", "00000000000100"));
        assert_eq!(Equal, compare_digits("000100", "0100"));

        assert_eq!(Equal, compare_digits("", "00"));
        assert_eq!(Less, compare_digits("", "001"));
        assert_eq!(Greater, compare_digits("001", ""));

        assert_eq!(Less, compare_digits("1", "2"));
        assert_eq!(Greater, compare_digits("2", "1"));

        assert_eq!(Less, compare_digits("01", "02"));
        assert_eq!(Greater, compare_digits("02", "01"));

        assert_eq!(Less, compare_digits("10", "20"));
        assert_eq!(Greater, compare_digits("20", "10"));

        assert_eq!(Less, compare_digits("11", "12"));
        assert_eq!(Greater, compare_digits("12", "11"));

        assert_eq!(Less, compare_digits("1", "10"));
        assert_eq!(Greater, compare_digits("10", "1"));
    }

    #[test]
    fn test_compare_non_digits() {
        use std::cmp::Ordering::*;
        use compare_non_digits;

        assert_eq!(Equal, compare_non_digits("a", "a"));
        assert_eq!(Equal, compare_non_digits("Z", "Z"));
        assert_eq!(Equal, compare_non_digits("~", "~"));
        assert_eq!(Equal, compare_non_digits("-", "-"));

        assert_eq!(Less, compare_non_digits("a", "b"));
    }

    #[test]
    fn simple() {
        assert_eq!(Ordering::Less, compare_versions("3.0", "3.1"));
        assert_eq!(Ordering::Greater, compare_versions("3.1", "3.0"));
        assert_eq!(Ordering::Equal, compare_versions("3.0", "3.0"));
    }
}
