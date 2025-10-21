use chrono::{Local, Timelike};
/// Parse time string in HH:MM format or named time (morning, noon, etc.)
/// Accepts flexible formats:
/// - Named times: "morning", "noon", "evening", etc.
/// - HH:MM format: "08:00", "8:00", "8:5" (with or without leading zeros)
/// - Hour only: "8", "08" (defaults to :00)
pub fn parse_time(time_str: &str) -> Option<(u32, u32)> {
    let trimmed = time_str.trim();

    // First, try to parse named times (case-insensitive)
    let time_lower = trimmed.to_lowercase();
    let named_time = match time_lower.as_str() {
        "morning" | "breakfast" => Some((8, 0)),
        "midmorning" | "mid-morning" => Some((10, 0)),
        "noon" | "midday" | "lunch" => Some((12, 0)),
        "afternoon" => Some((15, 0)),
        "evening" | "dinner" => Some((18, 0)),
        "night" | "bedtime" => Some((21, 0)),
        "midnight" => Some((0, 0)),
        _ => None,
    };

    if let Some(time) = named_time {
        return Some(time);
    }

    // Try to parse HH:MM format (or just H:MM, HH:M, H:M)
    if trimmed.contains(':') {
        let parts: Vec<&str> = trimmed.split(':').collect();
        if parts.len() != 2 {
            return None;
        }

        let hour = parts[0].trim().parse::<u32>().ok()?;
        let minute = parts[1].trim().parse::<u32>().ok()?;

        if hour >= 24 || minute >= 60 {
            return None;
        }

        return Some((hour, minute));
    }

    // Try to parse as just an hour (e.g., "8" means "08:00")
    if let Ok(hour) = trimmed.parse::<u32>() {
        if hour >= 24 {
            return None;
        }
        return Some((hour, 0));
    }

    None
}

/// Check if current time is at or past the scheduled time
pub fn is_time_due(scheduled_time: &str) -> bool {
    let Some((scheduled_hour, scheduled_min)) = parse_time(scheduled_time) else {
        return false;
    };

    let now = Local::now();
    let current_hour = now.hour();
    let current_min = now.minute();

    // Check if current time >= scheduled time
    current_hour > scheduled_hour
        || (current_hour == scheduled_hour && current_min >= scheduled_min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_time_named() {
        assert_eq!(parse_time("morning"), Some((8, 0)));
        assert_eq!(parse_time("MORNING"), Some((8, 0)));
        assert_eq!(parse_time("breakfast"), Some((8, 0)));
        assert_eq!(parse_time("noon"), Some((12, 0)));
        assert_eq!(parse_time("midday"), Some((12, 0)));
        assert_eq!(parse_time("lunch"), Some((12, 0)));
        assert_eq!(parse_time("afternoon"), Some((15, 0)));
        assert_eq!(parse_time("evening"), Some((18, 0)));
        assert_eq!(parse_time("dinner"), Some((18, 0)));
        assert_eq!(parse_time("night"), Some((21, 0)));
        assert_eq!(parse_time("bedtime"), Some((21, 0)));
        assert_eq!(parse_time("midnight"), Some((0, 0)));
    }

    #[test]
    fn test_parse_time_hhmm_format() {
        // Standard format
        assert_eq!(parse_time("08:00"), Some((8, 0)));
        assert_eq!(parse_time("14:30"), Some((14, 30)));
        assert_eq!(parse_time("23:59"), Some((23, 59)));
        assert_eq!(parse_time("00:00"), Some((0, 0)));

        // Without leading zeros
        assert_eq!(parse_time("8:00"), Some((8, 0)));
        assert_eq!(parse_time("8:5"), Some((8, 5)));
        assert_eq!(parse_time("8:30"), Some((8, 30)));

        // With whitespace
        assert_eq!(parse_time(" 8:00 "), Some((8, 0)));
        assert_eq!(parse_time("  14:30  "), Some((14, 30)));
    }

    #[test]
    fn test_parse_time_hour_only() {
        assert_eq!(parse_time("8"), Some((8, 0)));
        assert_eq!(parse_time("14"), Some((14, 0)));
        assert_eq!(parse_time("0"), Some((0, 0)));
        assert_eq!(parse_time("23"), Some((23, 0)));
        assert_eq!(parse_time(" 8 "), Some((8, 0)));
    }

    #[test]
    fn test_parse_time_invalid() {
        // Invalid hours
        assert_eq!(parse_time("24:00"), None);
        assert_eq!(parse_time("25:00"), None);
        assert_eq!(parse_time("24"), None);

        // Invalid minutes
        assert_eq!(parse_time("8:60"), None);
        assert_eq!(parse_time("8:99"), None);

        // Invalid formats
        assert_eq!(parse_time("garbage"), None);
        assert_eq!(parse_time("8:30:00"), None);
        assert_eq!(parse_time("abc:def"), None);
        assert_eq!(parse_time(""), None);
        assert_eq!(parse_time(":30"), None);
        assert_eq!(parse_time("8:"), None);
    }

    #[test]
    fn test_parse_time_edge_cases() {
        // Named times with variations
        assert_eq!(parse_time("mid-morning"), Some((10, 0)));
        assert_eq!(parse_time("MID-MORNING"), Some((10, 0)));

        // Boundary values
        assert_eq!(parse_time("0:0"), Some((0, 0)));
        assert_eq!(parse_time("23:59"), Some((23, 59)));
    }
}
