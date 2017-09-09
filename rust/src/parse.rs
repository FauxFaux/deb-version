named!(
    epoch(&str) -> &str,
    terminated!(
        take_while1_s!(|chr: char| chr.is_digit(10)),
        tag_s!(":")
));

fn alpha_sym(chr: char) -> bool {
    '.' == chr
        || (chr >= 'a' && chr <= 'z')
        || (chr >= 'A' && chr <= 'Z')
        || '+' == chr
        || '~' == chr
        || '-' == chr
}

named!(
    version(&str) -> Vec<(&str, &str)>,
    many1!(
        // TODO: is it a problem that this can match empty?
        pair!(
            take_while_s!(|chr: char| alpha_sym(chr)),
            take_while_s!(|chr: char| chr.is_digit(10))
        )
    )
);

named!(
    deb_version(&str) -> (
        Option<&str>,
        Vec<(&str, &str)>,
        Option<Vec<(&str, &str)> >
    ),
    tuple!(
        opt!(epoch),
        version,
        opt!(
            preceded!(
                tag_s!("-"),
                version
            )
        )
    )
);

named!(de(&str)  -> (
        Option<&str>,
        Vec<(&str, &str)>,
        Option<Vec<(&str, &str)> >
    ), dbg!(deb_version));

#[cfg(test)]
mod tests {
    use super::epoch;
    use super::version;
    use super::deb_version;
    use nom::IResult::Done;

    #[test]
    fn test_epoch() {
        assert!(epoch("17").is_incomplete());
        assert_eq!(Done("17", "1"), epoch("1:17"));
        assert_eq!(Done("12", "17"), epoch("17:12"));
    }

    #[test]
    fn test_version() {
        assert_eq!(Done("", vec![("", "1"), (".", "17")]), version("1.17"));
    }

    #[test]
    fn test_deb_version() {
        assert_eq!(Done("", (
            Some("1"),
            vec![("", "2"), (".", "3")],
            Some(vec![("", "4")])
        )),
           super::de("1:2.3-4"));
    }
}