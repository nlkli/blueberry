use time::format_description::well_known::Rfc3339;

pub fn format_rfc3339_to_unix_timestamp(s: &str) -> u64 {
    time::OffsetDateTime::parse(s, &Rfc3339)
        .map(|v| v.unix_timestamp())
        .unwrap_or_default() as u64
}

pub fn unix_timestamp_to_rfc3339_format(ts: u64) -> String {
    time::UtcDateTime::from_unix_timestamp(ts as i64)
        .map(|v| v.format(&Rfc3339).unwrap_or_default())
        .unwrap_or_default()
}
