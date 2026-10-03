//! Fails when a certificate embedded for the distribution protocol expires within 90 days,
//! to renew them in time (see `melodium-certs/README.md`).

use std::time::{SystemTime, UNIX_EPOCH};

const WARNING_DAYS: i64 = 90;

/// Splits a DER element into its tag, its content, and what follows it.
fn element(der: &[u8]) -> (u8, &[u8], &[u8]) {
    let tag = der[0];
    let (length, header) = if der[1] < 0x80 {
        (der[1] as usize, 2)
    } else {
        let bytes = (der[1] & 0x7f) as usize;
        let length = der[2..2 + bytes]
            .iter()
            .fold(0usize, |length, byte| (length << 8) | *byte as usize);
        (length, 2 + bytes)
    };
    (tag, &der[header..header + length], &der[header + length..])
}

/// Days from 1970-01-01 to the given date.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146097 + day_of_era - 719468
}

/// Gives the expiry of a certificate, as days since 1970-01-01 and as written.
fn not_after(certificate: &[u8]) -> (i64, String) {
    let (_, certificate, _) = element(certificate);
    let (_, to_be_signed, _) = element(certificate);
    // The version is optional, tagged [0].
    let (tag, _, after_version) = element(to_be_signed);
    let mut rest = if tag == 0xa0 {
        after_version
    } else {
        to_be_signed
    };
    // Serial number, signature algorithm and issuer.
    for _ in 0..3 {
        rest = element(rest).2;
    }
    let (_, validity, _) = element(rest);
    let (_, _not_before, rest) = element(validity);
    let (tag, not_after, _) = element(rest);

    let text = String::from_utf8(not_after.to_vec()).unwrap();
    let (year, date) = match tag {
        // UTCTime, YYMMDDHHMMSSZ
        0x17 => {
            let year: i64 = text[0..2].parse().unwrap();
            (
                if year < 50 { 2000 + year } else { 1900 + year },
                &text[2..],
            )
        }
        // GeneralizedTime, YYYYMMDDHHMMSSZ
        0x18 => (text[0..4].parse().unwrap(), &text[4..]),
        tag => panic!("unexpected time tag {tag:#x}"),
    };
    let month = date[0..2].parse().unwrap();
    let day = date[2..4].parse().unwrap();
    (
        days_from_civil(year, month, day),
        format!("{year}-{month:02}-{day:02}"),
    )
}

#[test]
fn embedded_certificates_do_not_expire_within_90_days() {
    let today = (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        / 86400) as i64;

    let mut expiring = Vec::new();
    for (file, mut pem) in [
        (
            "melodium-certs/melodium-ca.pem",
            melodium_certs::ROOT_CERTIFICATE,
        ),
        (
            "melodium-distribution/melodium-chain.pem",
            &include_bytes!("../melodium-chain.pem")[..],
        ),
    ] {
        let certificates = rustls_pemfile::certs(&mut pem)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(!certificates.is_empty(), "no certificate in {file}");
        for (position, certificate) in certificates.iter().enumerate() {
            let (expiry, date) = not_after(certificate);
            if expiry - today <= WARNING_DAYS {
                expiring.push(format!(
                    "certificate {} of {file} expires on {date}",
                    position + 1
                ));
            }
        }
    }

    assert!(
        expiring.is_empty(),
        "renew them, see melodium-certs/README.md:\n{}",
        expiring.join("\n")
    );
}

#[test]
fn days_are_counted_from_1970() {
    assert_eq!(days_from_civil(1970, 1, 1), 0);
    assert_eq!(days_from_civil(2000, 3, 1), 11017);
    assert_eq!(days_from_civil(2026, 12, 21), 20808);
}

#[test]
fn expiry_dates_are_read_from_both_time_formats() {
    for (mut pem, expected) in [
        // Before 2050, written as UTCTime.
        (
            &include_bytes!("fixtures/expiry-2049.crt")[..],
            "2049-05-31",
        ),
        // From 2050, written as GeneralizedTime.
        (
            &include_bytes!("fixtures/expiry-2099.crt")[..],
            "2099-05-31",
        ),
    ] {
        let certificate = rustls_pemfile::certs(&mut pem).next().unwrap().unwrap();
        let (days, date) = not_after(&certificate);
        assert_eq!(date, expected);
        let (year, month, day) = (
            date[0..4].parse().unwrap(),
            date[5..7].parse().unwrap(),
            date[8..10].parse().unwrap(),
        );
        assert_eq!(days, days_from_civil(year, month, day));
    }
}
