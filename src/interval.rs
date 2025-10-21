/// Parse medication frequency/interval into number of days between doses
///
/// Supported formats:
/// - "daily" -> 1 day
/// - "weekly" -> 7 days
/// - "monthly" -> 30 days
/// - "every X days" -> X days
/// - "every X day" -> X days
/// - "twice daily", "3 times daily" -> 1 day (multiple doses per day treated as daily)
pub fn parse_interval_to_days(interval: &str) -> Option<u32> {
    let lower = interval.trim().to_lowercase();

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
        assert_eq!(parse_interval_to_days("garbage"), Some(1)); // defaults to daily for safety
    }
}
