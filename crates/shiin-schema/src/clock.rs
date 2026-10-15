//! Timestamps and identifiers.
//!
//! These are the only impure parts of this crate. They are kept here so that every
//! other crate takes time and identity as inputs rather than reading the clock,
//! which is what keeps the engine replayable.

use std::fmt::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

/// The current time as an RFC 3339 timestamp in UTC, to the second.
#[must_use]
pub fn now_rfc3339() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    rfc3339_from_unix_seconds(seconds)
}

/// Seconds since the Unix epoch.
#[must_use]
pub fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

/// Formats Unix seconds as an RFC 3339 timestamp in UTC.
#[must_use]
pub fn rfc3339_from_unix_seconds(seconds: u64) -> String {
    let days = i64::try_from(seconds / 86_400).unwrap_or(0);
    let time_of_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = time_of_day / 3_600;
    let minute = (time_of_day % 3_600) / 60;
    let second = time_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Converts days since the Unix epoch to a civil date.
///
/// This is Howard Hinnant's `civil_from_days` algorithm, which is exact and needs
/// no lookup tables.
fn civil_from_days(days: i64) -> (i64, u64, u64) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = u64::try_from(day_of_year - (153 * shifted_month + 2) / 5 + 1).unwrap_or(1);
    let month = u64::try_from(if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    })
    .unwrap_or(1);
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// A random version 4 UUID, as required by `AR-002`.
///
/// Returns an empty identifier if OS randomness fails. Action validation and state
/// writers reject that sentinel rather than authorizing with a fabricated identifier.
#[must_use]
pub fn new_id() -> String {
    let mut bytes = [0_u8; 16];
    if getrandom::fill(&mut bytes).is_err() {
        return String::new();
    }
    // Version 4 in the high nibble of byte 6, RFC variant in byte 8.
    bytes[6] = (bytes[6] & 0x0F) | 0x40;
    bytes[8] = (bytes[8] & 0x3F) | 0x80;

    let hex = bytes
        .iter()
        .fold(String::with_capacity(32), |mut accumulator, byte| {
            let _ = write!(accumulator, "{byte:02x}");
            accumulator
        });
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_instants_format_correctly() {
        assert_eq!(rfc3339_from_unix_seconds(0), "1970-01-01T00:00:00Z");
        assert_eq!(
            rfc3339_from_unix_seconds(1_760_000_000),
            "2025-10-09T08:53:20Z"
        );
    }

    #[test]
    fn identifiers_look_like_uuid_v4() {
        let id = new_id();
        assert_eq!(id.len(), 36);
        assert_eq!(id.as_bytes()[14], b'4', "version nibble is 4: {id}");
        assert!(matches!(id.as_bytes()[19], b'8' | b'9' | b'a' | b'b'));
        assert_ne!(new_id(), new_id(), "identifiers are not repeated");
    }
}
