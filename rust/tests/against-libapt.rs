extern crate apt_pkg_native;
extern crate deb_version;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quickcheck;
extern crate regex;

use std::cmp::Ordering;

use deb_version::compare_versions;
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
        && self.version.find(|x: char| !VALID_CHARS.contains(x)).is_none()
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
                    version.push(*g.choose(VALID_CHARS.as_bytes()).unwrap() as char);
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
        Box::new(chars.shrink().map(|x| {
            VersionString { version: x.into_iter().collect::<String>() }
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
