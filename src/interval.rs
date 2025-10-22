/// Parse medication frequency/interval into number of days between doses
///
/// Supported formats:
/// - "daily" -> 1 day
/// - "weekly" -> 7 days
/// - "monthly" -> 30 days
/// - "every X days" -> X days
/// - "every X day" -> X days
/// - "twice daily", "3 times daily" -> 1 day (multiple doses per day treated as daily)
/// - "prn", "as needed" -> None (no interval, take as needed)
pub fn parse_interval_to_days(interval: &str) -> Option<u32> {
    let lower = interval.trim().to_lowercase();

    // Handle PRN (as-needed) medications - no interval checking
    match lower.as_str() {
        "prn" | "as needed" | "as-needed" | "asneeded" | "when needed" => return None,
        _ => {}
    }

    // Handle common named intervals
    match lower.as_str() {
        "daily" | "every day" => return Some(1),
        "weekly" | "every week" => return Some(7),
        "monthly" | "every month" => return Some(30),
        _ => {}
    }

    // Handle "every X days" or "every X day" BEFORE checking for generic "day" mentions
    if lower.starts_with("every ") {
        let parts: Vec<&str> = lower.split_whitespace().collect();
        if parts.len() >= 3 {
            // "every X days" or "every X day"
            if let Ok(num) = parts[1].parse::<u32>() {
                if parts[2].starts_with("day") || parts[2].starts_with("week") {
                    if parts[2].starts_with("week") {
                        return Some(num * 7);
                    } else {
                        return Some(num);
                    }
                }
            }
        }
    }

    // Handle "twice daily", "3 times daily" etc - these are still daily medications
    if lower.contains("daily") || lower.contains("day") {
        return Some(1);
    }

    // Default to daily if we can't parse it (safest option - more reminders rather than fewer)
    Some(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_intervals() {
        assert_eq!(parse_interval_to_days("daily"), Some(1));
        assert_eq!(parse_interval_to_days("Daily"), Some(1));
        assert_eq!(parse_interval_to_days("weekly"), Some(7));
        assert_eq!(parse_interval_to_days("monthly"), Some(30));
        assert_eq!(parse_interval_to_days("every 3 days"), Some(3));
        assert_eq!(parse_interval_to_days("every 2 weeks"), Some(14));
        assert_eq!(parse_interval_to_days("twice daily"), Some(1));
        assert_eq!(parse_interval_to_days("3 times daily"), Some(1));
        assert_eq!(parse_interval_to_days("prn"), None);
        assert_eq!(parse_interval_to_days("PRN"), None);
        assert_eq!(parse_interval_to_days("as needed"), None);
        assert_eq!(parse_interval_to_days("as-needed"), None);
        assert_eq!(parse_interval_to_days("when needed"), None);
        assert_eq!(parse_interval_to_days("garbage"), Some(1)); // defaults to daily for safety
    }

    #[test]
    fn test_prn_edge_cases() {
        // Various PRN formats
        assert_eq!(parse_interval_to_days("prn"), None);
        assert_eq!(parse_interval_to_days("PRN"), None);
        assert_eq!(parse_interval_to_days("PrN"), None);
        assert_eq!(parse_interval_to_days("  prn  "), None); // with spaces
        assert_eq!(parse_interval_to_days("as needed"), None);
        assert_eq!(parse_interval_to_days("AS NEEDED"), None);
        assert_eq!(parse_interval_to_days("As-Needed"), None);
        assert_eq!(parse_interval_to_days("asneeded"), None);
        assert_eq!(parse_interval_to_days("when needed"), None);
        assert_eq!(parse_interval_to_days("WHEN NEEDED"), None);
    }

    #[test]
    fn test_interval_boundaries() {
        // Large numbers
        assert_eq!(parse_interval_to_days("every 30 days"), Some(30));
        assert_eq!(parse_interval_to_days("every 365 days"), Some(365));

        // Single day/week variations
        assert_eq!(parse_interval_to_days("every 1 day"), Some(1));
        assert_eq!(parse_interval_to_days("every 1 week"), Some(7));

        // Edge cases with "day" in string
        assert_eq!(parse_interval_to_days("daily medication"), Some(1));
        assert_eq!(parse_interval_to_days("take during the day"), Some(1)); // contains "day"
    }

    #[test]
    fn test_interval_multiple_doses() {
        // Multiple doses per day all map to daily (interval = 1)
        assert_eq!(parse_interval_to_days("twice daily"), Some(1));
        assert_eq!(parse_interval_to_days("2 times daily"), Some(1));
        assert_eq!(parse_interval_to_days("three times daily"), Some(1));
        assert_eq!(parse_interval_to_days("4 times a day"), Some(1));
        assert_eq!(parse_interval_to_days("every 8 hours"), Some(1)); // contains "day" fallthrough
    }
}
