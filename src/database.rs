use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Medication {
    pub name: String,
    pub dose: String,
    pub time_of_day: String,
    pub medication_frequency: String,
    pub taken: bool,
    pub taken_at: String,
    /// Date of last dose in YYYY-MM-DD format (for interval tracking)
    #[serde(default)]
    pub last_dose_date: String,
    pub notes: Option<String>,
}

pub fn get_data_file() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".pharm.json")
}

pub fn load_medications() -> Vec<Medication> {
    let file_path = get_data_file();
    if !file_path.exists() {
        return Vec::new();
    }

    let contents = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: Failed to read medications file: {}", e);
            eprintln!(
                "Using empty medication list. Check file permissions on: {}",
                file_path.display()
            );
            return Vec::new();
        }
    };

    match serde_json::from_str(&contents) {
        Ok(meds) => meds,
        Err(e) => {
            eprintln!("WARNING: Medications file is corrupted! Error: {}", e);
            eprintln!("File location: {}", file_path.display());
            eprintln!("Creating backup at: {}.corrupted", file_path.display());

            // Create backup of corrupted file
            let backup_path = file_path.with_extension("json.corrupted");
            if let Err(backup_err) = fs::copy(&file_path, &backup_path) {
                eprintln!("Failed to create backup: {}", backup_err);
            } else {
                eprintln!("Backup created successfully.");
            }

            eprintln!("Starting with empty medication list.");
            Vec::new()
        }
    }
}

pub fn save_medications(meds: &[Medication]) {
    let file_path = get_data_file();

    let json = match serde_json::to_string_pretty(meds) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Error: Failed to serialize medications: {}", e);
            return;
        }
    };

    // Atomic write: write to temp file, then rename
    let temp_path = file_path.with_extension("json.tmp");

    if let Err(e) = fs::write(&temp_path, &json) {
        eprintln!("Error: Failed to write temporary file: {}", e);
        return;
    }

    // Rename is atomic on POSIX systems
    if let Err(e) = fs::rename(&temp_path, &file_path) {
        eprintln!("Error: Failed to save medications file: {}", e);
        // Clean up temp file
        let _ = fs::remove_file(&temp_path);
    }
}

pub fn add_medication(
    name: String,
    dose: String,
    time: String,
    interval: String,
    notes: Option<String>,
) {
    // Validate inputs
    if name.trim().is_empty() {
        eprintln!("Error: Medication name cannot be empty!");
        return;
    }

    if dose.trim().is_empty() {
        eprintln!("Error: Dose cannot be empty!");
        return;
    }

    if interval.trim().is_empty() {
        eprintln!("Error: Interval cannot be empty!");
        return;
    }

    // Validate that time is parseable
    if crate::time::parse_time(&time).is_none() {
        eprintln!("Error: Invalid time format '{}'", time);
        eprintln!("Valid formats:");
        eprintln!("  - Named times: 'morning', 'noon', 'evening', 'bedtime'");
        eprintln!("  - Time format: '8:00', '08:30', '14:15'");
        eprintln!("  - Hour only: '8', '14' (defaults to :00)");
        return;
    }

    let mut meds = load_medications();

    // Check if medication already exists (case-insensitive)
    let name_lower = name.to_lowercase();
    if meds.iter().any(|m| m.name.to_lowercase() == name_lower) {
        eprintln!("Error: Medication '{}' already exists!", name);
        return;
    }

    let med = Medication {
        name: name.clone(),
        dose,
        time_of_day: time,
        medication_frequency: interval,
        taken: false,
        taken_at: String::new(),
        last_dose_date: String::new(),
        notes,
    };

    meds.push(med);
    save_medications(&meds);
    println!("Added medication: {}", name);
}

pub fn remove_medication(name: String) {
    let mut meds = load_medications();
    let original_len = meds.len();
    let name_lower = name.to_lowercase();

    meds.retain(|m| m.name.to_lowercase() != name_lower);

    if meds.len() == original_len {
        println!("Medication '{}' not found!", name);
    } else {
        save_medications(&meds);
        println!("Removed medication: {}", name);
    }
}

pub fn list_medications() {
    let meds = load_medications();

    if meds.is_empty() {
        println!("No medications found.");
        return;
    }

    println!("\nMedications:");
    println!("{}", "=".repeat(60));

    for med in meds {
        println!("\n{}", med.name);
        println!("  Dose:     {}", med.dose);
        println!("  Time:     {}", med.time_of_day);
        println!("  Interval: {}", med.medication_frequency);
        println!("  Taken:    {}", if med.taken { "✓" } else { "✗" });
        println!("  Taken At: {}", med.taken_at);
        if let Some(notes) = med.notes {
            println!("  Notes:    {}", notes);
        }
    }
    println!();
}
pub fn take_medication(name: String) {
    let mut meds = load_medications();
    let mut found = false;
    let name_lower = name.to_lowercase();
    let now = chrono::Local::now();
    let now_str = now.format("%H:%M:%S - %Y/%m/%d").to_string();
    let today = now.format("%Y-%m-%d").to_string();

    for med in meds.iter_mut() {
        if med.name.to_lowercase() == name_lower {
            match med.taken {
                true => {
                    println!("Medication already marked as taken at {}", med.taken_at);
                    return;
                }
                false => {
                    med.taken = true;
                    med.taken_at = now_str.clone();
                    med.last_dose_date = today;
                    found = true;
                    break;
                }
            }
        }
    }
    if found {
        save_medications(&meds);
        println!("Marked '{}' as taken at {}", name, now_str);
    } else {
        println!("Medication '{}' not found!", name);
    }
}
pub fn untake_medication(name: String) {
    let mut meds = load_medications();
    let mut found = false;
    let name_lower = name.to_lowercase();

    for med in meds.iter_mut() {
        if med.name.to_lowercase() == name_lower {
            if !med.taken {
                println!("Medication '{}' is not currently marked as taken", med.name);
                return;
            }
            med.taken = false;
            med.taken_at = String::new();
            // Keep last_dose_date - it's still needed for interval tracking
            found = true;
            break;
        }
    }

    if found {
        save_medications(&meds);
        println!("Unmarked '{}' as taken", name);
    } else {
        println!("Medication '{}' not found!", name);
    }
}

pub fn take_all_medications() {
    let mut meds = load_medications();
    let now = chrono::Local::now();
    let now_str = now.format("%H:%M:%S - %Y/%m/%d").to_string();
    let today = now.format("%Y-%m-%d").to_string();

    if meds.is_empty() {
        println!("No medications to mark as taken.");
        return;
    }

    for med in meds.iter_mut() {
        match med.taken {
            true => {
                println!(
                    "Medication {} already marked as taken at {}",
                    med.name, med.taken_at
                );
            }
            false => {
                med.taken = true;
                med.taken_at = now_str.clone();
                med.last_dose_date = today.clone();
            }
        }
    }

    save_medications(&meds);
    println!("Marked all medications as taken at {}", now_str);
}

pub fn edit_medication(
    name: String,
    new_dose: Option<String>,
    new_time: Option<String>,
    new_freq: Option<String>,
    new_notes: Option<String>,
) {
    let mut meds = load_medications();
    let mut found = false;
    let name_lower = name.to_lowercase();

    // Validate new time if provided
    if let Some(ref time) = new_time {
        if crate::time::parse_time(time).is_none() {
            eprintln!("Error: Invalid time format '{}'", time);
            eprintln!("Valid formats:");
            eprintln!("  - Named times: 'morning', 'noon', 'evening', 'bedtime'");
            eprintln!("  - Time format: '8:00', '08:30', '14:15'");
            eprintln!("  - Hour only: '8', '14' (defaults to :00)");
            return;
        }
    }

    // Validate new dose if provided
    if let Some(ref dose) = new_dose {
        if dose.trim().is_empty() {
            eprintln!("Error: Dose cannot be empty!");
            return;
        }
    }

    // Validate new frequency if provided
    if let Some(ref freq) = new_freq {
        if freq.trim().is_empty() {
            eprintln!("Error: Frequency cannot be empty!");
            return;
        }
    }

    for med in meds.iter_mut() {
        if med.name.to_lowercase() == name_lower {
            let mut changes = Vec::new();

            if let Some(dose) = new_dose {
                med.dose = dose.clone();
                changes.push(format!("dose -> {}", dose));
            }

            if let Some(time) = new_time {
                med.time_of_day = time.clone();
                changes.push(format!("time -> {}", time));
            }

            if let Some(freq) = new_freq {
                med.medication_frequency = freq.clone();
                changes.push(format!("frequency -> {}", freq));
            }

            if let Some(notes) = new_notes {
                if notes.is_empty() {
                    med.notes = None;
                    changes.push("notes -> (cleared)".to_string());
                } else {
                    med.notes = Some(notes.clone());
                    changes.push(format!("notes -> {}", notes));
                }
            }

            if changes.is_empty() {
                println!("No changes specified for '{}'", med.name);
                return;
            }

            found = true;
            println!("Updated '{}': {}", med.name, changes.join(", "));
            break;
        }
    }

    if found {
        save_medications(&meds);
    } else {
        println!("Medication '{}' not found!", name);
    }
}

/// Reset medications to untaken status if their interval has passed (called at midnight by daemon)
pub fn reset_all_medications() {
    let mut meds = load_medications();

    if meds.is_empty() {
        return;
    }

    let today_date = chrono::Local::now().date_naive();
    let mut reset_count = 0;

    for med in meds.iter_mut() {
        if !med.taken {
            continue; // Skip if not taken
        }

        // Parse interval to determine if we should reset
        let interval_days = crate::interval::parse_interval_to_days(&med.medication_frequency).unwrap_or(1);

        // Parse last dose date
        let should_reset = if med.last_dose_date.is_empty() {
            // No last dose date, reset to be safe
            true
        } else if let Ok(last_dose) = chrono::NaiveDate::parse_from_str(&med.last_dose_date, "%Y-%m-%d") {
            let days_since_dose = (today_date - last_dose).num_days();
            days_since_dose >= interval_days as i64
        } else {
            // Can't parse date, reset to be safe
            true
        };

        if should_reset {
            med.taken = false;
            med.taken_at = String::new();
            // Don't clear last_dose_date - we need it for interval tracking
            reset_count += 1;
        }
    }

    if reset_count > 0 {
        save_medications(&meds);
    }
}
