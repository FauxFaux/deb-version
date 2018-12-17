extern crate apt_pkg_native;
extern crate deb_version;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quickcheck;
extern crate rand;
extern crate regex;

use std::cmp::Ordering;

use deb_version::compare_versions;
use rand::prelude::SliceRandom;
use rand::Rng;
use regex::Regex;

lazy_static! {
    static ref BASIC_VALIDATE: Regex = Regex::new("^(?:(?:(?:\\d+:).+)|(?:[^:]+))$").unwrap();
}

fn libapt(left: &str, right: &str) -> Ordering {
    let cache = apt_pkg_native::Cache::get_singleton();
    cache.compare_versions(left, right)
}

const VALID_CHARS: &str = "abc+xyz.ABC-XYZ~012:789";

#[derive(Clone, Debug)]
struct VersionString {
    pub version: String,
}

impl VersionString {
    fn valid(&self) -> bool {
        !self.version.is_empty()
            && self
                .version
                .find(|x: char| !VALID_CHARS.contains(x))
                .is_none()
            && BASIC_VALIDATE.is_match(self.version.as_ref())
    }
}

/// Very similar to quickcheck's impl for String, but with a limited character set.
/// The shrinking does not do the right thing, it manages to introduce control characters
/// in the output; I don't really get why. Maybe it's making new elements? I don't think
/// it's supposed to be doing that. I'm fixing it by discarding them at test time. :(
impl quickcheck::Arbitrary for VersionString {
    fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
        let size = {
            let s = g.size();
            g.gen_range(1, s)
        };

        loop {
            let made = {
                let mut version = String::with_capacity(size);
                for _ in 0..size {
                    version.push(*VALID_CHARS.as_bytes().choose(g).unwrap() as char);
                }

                VersionString { version }
            };
            if made.valid() {
                return made;
            }
        }
    }

    fn shrink(&self) -> Box<Iterator<Item = Self>> {
        let chars: Vec<char> = self.version.chars().collect();
        Box::new(chars.shrink().map(|x| VersionString {
            version: x.into_iter().collect::<String>(),
        }))
    }
}

quickcheck! {
    fn versions_eq(left: VersionString, right: VersionString) -> quickcheck::TestResult {
        if !left.valid() || !right.valid() {
            return quickcheck::TestResult::discard();
        }

//        println!("{} - {}", left.version, right.version);

        quickcheck::TestResult::from_bool(
            libapt(&left.version, &right.version) == compare_versions(&left.version, &right.version)
        )
    }
}

#[test]
fn sort_everything() {
    let everything = include_str!("../../all-debian-versions.lst").trim();
    let mut everything: Vec<&str> = everything.split('\n').collect();
    everything.shuffle(&mut rand::thread_rng());
    assert!(everything.len() > 30_000);

    // shuffle then sort_by(_stable) as 1.00.00 == 1.0.0, which breaks this test
    // but hopefully not the real world

    let mut apt = everything.clone();
    apt.sort_by(|left, right| libapt(left, right));

    let mut us: Vec<&str> = everything.clone();
    us.sort_by(|left, right| compare_versions(left, right));

    assert_eq!(apt.len(), us.len());

    for (line, (apt, us)) in apt.iter().zip(us.iter()).enumerate() {
        if apt != us {
            panic!("varied at line {}: {} {}", line, apt, us);
        }
    }
}
