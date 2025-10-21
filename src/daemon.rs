use chrono::{Datelike, Local, NaiveDate};
use notify_rust::Notification;
use std::collections::HashSet;
use std::thread;
use std::time::Duration;

use crate::database::{load_medications, reset_all_medications};
use crate::interval::parse_interval_to_days;
use crate::time::is_time_due;

/// Check if enough time has passed since last dose based on medication interval
fn is_medication_due_by_interval(last_dose_date: &str, interval_str: &str, today: &NaiveDate) -> bool {
    // If never taken, it's due
    if last_dose_date.is_empty() {
        return true;
    }

    // Parse last dose date
    let last_dose = match NaiveDate::parse_from_str(last_dose_date, "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => return true, // If we can't parse, assume it's due (safer)
    };

    // Parse interval to days
    let interval_days = parse_interval_to_days(interval_str).unwrap_or(1);

    // Calculate days since last dose
    let days_since_dose = (*today - last_dose).num_days();

    // Medication is due if enough days have passed
    days_since_dose >= interval_days as i64
}

pub fn run_daemon() {
    println!("Daemon started. Checking for medication reminders...");
    println!("Press Ctrl+C to stop.");

    // Track which medications we've already notified about today
    let mut notified_today: HashSet<String> = HashSet::new();
    let mut current_day = Local::now().day();

    loop {
        let now = Local::now();

        // Reset notifications and medication status at midnight
        if now.day() != current_day {
            notified_today.clear();
            current_day = now.day();
            println!(
                "[{}] New day detected - resetting all medications to untaken",
                now.format("%H:%M:%S")
            );
            reset_all_medications();
        }

        let meds = load_medications();
        let today_date = now.date_naive();

        for med in meds.iter() {
            // Clear notification flag if medication was taken
            if med.taken && notified_today.contains(&med.name) {
                notified_today.remove(&med.name);
            }

            // Check if medication is due by both time-of-day AND interval
            let time_is_due = is_time_due(&med.time_of_day);
            let interval_allows = is_medication_due_by_interval(
                &med.last_dose_date,
                &med.medication_frequency,
                &today_date
            );

            // Only notify for untaken medications that are:
            // 1. Past their scheduled time of day
            // 2. Haven't been taken too recently (interval check)
            // 3. Haven't been notified yet today
            if !med.taken && time_is_due && interval_allows && !notified_today.contains(&med.name) {
                let result = Notification::new()
                    .summary("Medication Reminder")
                    .body(&format!(
                        "Time to take: {} ({})\nScheduled for: {}",
                        med.name, med.dose, med.time_of_day
                    ))
                    .icon("medication")
                    .timeout(0) // Don't auto-dismiss
                    .show();

                if result.is_ok() {
                    notified_today.insert(med.name.clone());
                    println!(
                        "[{}] Reminder sent: {} - {}",
                        now.format("%H:%M:%S"),
                        med.name,
                        med.dose
                    );
                } else {
                    eprintln!(
                        "[{}] Failed to send notification for: {}",
                        now.format("%H:%M:%S"),
                        med.name
                    );
                }
            }
        }

        // Check every 60 seconds
        thread::sleep(Duration::from_secs(60));
    }
}
