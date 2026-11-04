//! Port of pacman's `version.c` (`alpm_pkg_vercmp`, `parseEVR`, `rpmvercmp`).
//!
//! The C code works on NUL-terminated bytes and uses the C-locale
//! `isalnum`/`isdigit`/`isalpha`, so this port works on bytes and treats only
//! ASCII letters and digits as segment characters.

use std::cmp::Ordering;

fn is_alnum(b: u8) -> bool {
    b.is_ascii_alphanumeric()
}

fn is_alpha(b: u8) -> bool {
    b.is_ascii_alphabetic()
}

/// Port of `rpmvercmp` from pacman (originally rpm 4.8.1).
pub fn rpmvercmp(a: &str, b: &str) -> Ordering {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a == b {
        return Ordering::Equal;
    }

    // `one`/`two` walk the strings; `ptr1`/`ptr2` mark the end of the previous
    // segment, which is how C measures separator-run lengths.
    let (mut one, mut two) = (0usize, 0usize);
    let (mut ptr1, mut ptr2) = (0usize, 0usize);

    // Same loop condition as pacman's `while (*one && *two)`.
    while one < a.len() && two < b.len() {
        while one < a.len() && !is_alnum(a[one]) {
            one += 1;
        }
        while two < b.len() && !is_alnum(b[two]) {
            two += 1;
        }

        // Ran to the end of either string: the loop is finished.
        if one >= a.len() || two >= b.len() {
            break;
        }

        // Differing separator-run lengths decide immediately.
        if (one - ptr1) != (two - ptr2) {
            return (one - ptr1).cmp(&(two - ptr2));
        }

        ptr1 = one;
        ptr2 = two;

        // Grab the first completely numeric or completely alpha segment.
        let isnum = a[ptr1].is_ascii_digit();
        if isnum {
            while ptr1 < a.len() && a[ptr1].is_ascii_digit() {
                ptr1 += 1;
            }
            while ptr2 < b.len() && b[ptr2].is_ascii_digit() {
                ptr2 += 1;
            }
        } else {
            while ptr1 < a.len() && is_alpha(a[ptr1]) {
                ptr1 += 1;
            }
            while ptr2 < b.len() && is_alpha(b[ptr2]) {
                ptr2 += 1;
            }
        }

        let mut seg1 = &a[one..ptr1];
        let mut seg2 = &b[two..ptr2];

        // Segment types differ (one numeric, the other empty of that type):
        // numeric is newer than alpha.
        if seg2.is_empty() {
            return if isnum { Ordering::Greater } else { Ordering::Less };
        }

        if isnum {
            // Strip leading zeros, then the longer number wins.
            while let [b'0', rest @ ..] = seg1 {
                seg1 = rest;
            }
            while let [b'0', rest @ ..] = seg2 {
                seg2 = rest;
            }
            match seg1.len().cmp(&seg2.len()) {
                Ordering::Equal => {}
                other => return other,
            }
        }

        match seg1.cmp(seg2) {
            Ordering::Equal => {}
            other => return other,
        }

        one = ptr1;
        two = ptr2;
    }

    if one >= a.len() && two >= b.len() {
        return Ordering::Equal;
    }

    // Final showdown: a remaining alpha string never beats an empty string.
    let one_next = a.get(one).copied();
    let two_next = b.get(two).copied();
    let one_empty = one_next.is_none();
    let two_is_alpha = two_next.is_some_and(is_alpha);
    let one_is_alpha = one_next.is_some_and(is_alpha);
    if (one_empty && !two_is_alpha) || one_is_alpha {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

/// Splits `[epoch:]version[-release]` the way pacman's `parseEVR` does.
///
/// The epoch defaults to `"0"` when absent (pacman differs from RPM here) and
/// the release is the text after the last `-`.
fn parse_evr(evr: &str) -> (&str, &str, Option<&str>) {
    let digits = evr.bytes().take_while(u8::is_ascii_digit).count();
    let (epoch, mut version) = if evr.as_bytes().get(digits) == Some(&b':') {
        let epoch = &evr[..digits];
        let epoch = if epoch.is_empty() { "0" } else { epoch };
        (epoch, &evr[digits + 1..])
    } else {
        ("0", evr)
    };

    let release = match version.rfind('-') {
        Some(idx) => {
            let release = &version[idx + 1..];
            version = &version[..idx];
            Some(release)
        }
        None => None,
    };
    (epoch, version, release)
}

/// Port of `alpm_pkg_vercmp`: compares epoch, then version, then release (the
/// release only when both sides have one).
pub fn vercmp(a: &str, b: &str) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }
    let (epoch1, ver1, rel1) = parse_evr(a);
    let (epoch2, ver2, rel2) = parse_evr(b);

    let ret = rpmvercmp(epoch1, epoch2)
        .then_with(|| rpmvercmp(ver1, ver2));
    match (ret, rel1, rel2) {
        (Ordering::Equal, Some(r1), Some(r2)) => rpmvercmp(r1, r2),
        _ => ret,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vectors from pacman's `test/util/vercmptest.sh` at the pinned commit
    /// 663941f. Each pair is also checked in reverse with the sign flipped.
    const OFFICIAL: &[(&str, &str, i32)] = &[
        ("1.5.0", "1.5.0", 0),
        ("1.5.1", "1.5.0", 1),
        ("1.5.1", "1.5", 1),
        ("1.5.0-1", "1.5.0-1", 0),
        ("1.5.0-1", "1.5.0-2", -1),
        ("1.5.0-1", "1.5.1-1", -1),
        ("1.5.0-2", "1.5.1-1", -1),
        ("1.5-1", "1.5.1-1", -1),
        ("1.5-2", "1.5.1-1", -1),
        ("1.5-2", "1.5.1-2", -1),
        ("1.5", "1.5-1", 0),
        ("1.5-1", "1.5", 0),
        ("1.1-1", "1.1", 0),
        ("1.0-1", "1.1", -1),
        ("1.1-1", "1.0", 1),
        ("1.5b-1", "1.5-1", -1),
        ("1.5b", "1.5", -1),
        ("1.5b-1", "1.5", -1),
        ("1.5b", "1.5.1", -1),
        ("1.0a", "1.0alpha", -1),
        ("1.0alpha", "1.0b", -1),
        ("1.0b", "1.0beta", -1),
        ("1.0beta", "1.0rc", -1),
        ("1.0rc", "1.0", -1),
        ("1.5.a", "1.5", 1),
        ("1.5.b", "1.5.a", 1),
        ("1.5.1", "1.5.b", 1),
        ("1.5.b-1", "1.5.b", 0),
        ("1.5-1", "1.5.b", -1),
        ("2.0", "2_0", 0),
        ("2.0_a", "2_0.a", 0),
        ("2.0a", "2.0.a", -1),
        ("2___a", "2_a", 1),
        ("0:1.0", "0:1.0", 0),
        ("0:1.0", "0:1.1", -1),
        ("1:1.0", "0:1.0", 1),
        ("1:1.0", "0:1.1", 1),
        ("1:1.0", "2:1.1", -1),
        ("1:1.0", "0:1.0-1", 1),
        ("1:1.0-1", "0:1.1-1", 1),
        ("0:1.0", "1.0", 0),
        ("0:1.0", "1.1", -1),
        ("0:1.1", "1.0", 1),
        ("1:1.0", "1.0", 1),
        ("1:1.0", "1.1", 1),
        ("1:1.1", "1.1", 1),
    ];

    /// Ordering chain from `vercmp(8)`, strictly ascending.
    const CHAIN: &[&str] = &[
        "1.0a", "1.0b", "1.0beta", "1.0p", "1.0pre", "1.0rc", "1.0", "1.0.a", "1.0.1",
    ];

    fn sign(o: Ordering) -> i32 {
        match o {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }

    #[test]
    fn official_pacman_vectors() {
        for &(a, b, want) in OFFICIAL {
            assert_eq!(sign(vercmp(a, b)), want, "vercmp {a} {b}");
        }
    }

    #[test]
    fn official_vectors_mirror_with_flipped_sign() {
        for &(a, b, want) in OFFICIAL {
            assert_eq!(sign(vercmp(b, a)), -want, "vercmp {b} {a}");
        }
    }

    #[test]
    fn vercmp_man_page_examples() {
        assert_eq!(vercmp("2.0", "2.0-13"), Ordering::Equal);
        assert_eq!(vercmp("4.34", "1:001"), Ordering::Less);
    }

    #[test]
    fn vercmp_man_page_chain_is_ascending() {
        for pair in CHAIN.windows(2) {
            assert_eq!(vercmp(pair[0], pair[1]), Ordering::Less, "{} < {}", pair[0], pair[1]);
        }
    }

    #[test]
    fn leading_zeros_and_long_numbers() {
        assert_eq!(vercmp("1.01", "1.1"), Ordering::Equal);
        assert_eq!(vercmp("1.10", "1.9"), Ordering::Greater);
        assert_eq!(vercmp("1.0000000000000000000002", "1.1"), Ordering::Greater);
    }

    #[test]
    fn release_only_compared_when_both_present() {
        assert_eq!(vercmp("1.0-1", "1.0"), Ordering::Equal);
        assert_eq!(vercmp("1.0-2", "1.0-10"), Ordering::Less);
    }

    #[test]
    fn real_package_versions() {
        assert_eq!(vercmp("7.1.0.r9.g54d9411-3", "7.1.0.r9.g54d9411-5"), Ordering::Less);
        assert_eq!(vercmp("6.1.0-1", "1:0.5-1"), Ordering::Less);
        assert_eq!(vercmp("2026.10.15-1", "2026.10.15-1"), Ordering::Equal);
    }
}
