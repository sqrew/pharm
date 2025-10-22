use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use chrono::TimeZone;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DoseRecord {
    pub timestamp: String, // Full datetime: "2025-10-21 08:30:15"
    pub dose: String,      // Dose at time of taking (in case it changes)
}

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
    /// Complete history of all doses taken
    #[serde(default)]
    pub history: Vec<DoseRecord>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MedicationDatabase {
    pub medications: Vec<Medication>,
    #[serde(default)]
    pub archived_medications: Vec<Medication>,
}

/// Returns the path to the medication database file.
///
/// Uses the `dirs` crate to reliably locate the home directory across platforms.
/// Falls back to `./.pharm.json` if no home directory is found.
pub fn get_data_file() -> PathBuf {
    // Use dirs crate for cross-platform home directory detection
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".pharm.json")
}

/// Loads the medication database from disk.
///
/// Handles automatic migration from the old format (just `Vec<Medication>`)
/// to the new format with archive support (`MedicationDatabase`).
///
/// If the file is corrupted, creates a backup and returns an empty database.
/// If the file doesn't exist, returns an empty database.
pub fn load_database() -> MedicationDatabase {
    let file_path = get_data_file();
    if !file_path.exists() {
        return MedicationDatabase {
            medications: Vec::new(),
            archived_medications: Vec::new(),
        };
    }

    let contents = match fs::read_to_string(&file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: Failed to read medications file: {}", e);
            eprintln!(
                "Using empty medication list. Check file permissions on: {}",
                file_path.display()
            );
            return MedicationDatabase {
                medications: Vec::new(),
                archived_medications: Vec::new(),
            };
        }
    };

    // Try to parse as new format first
    if let Ok(db) = serde_json::from_str::<MedicationDatabase>(&contents) {
        return db;
    }

    // Try to parse as old format (just Vec<Medication>) and migrate
    if let Ok(meds) = serde_json::from_str::<Vec<Medication>>(&contents) {
        eprintln!("Migrating medication database to new format with archive support...");
        let db = MedicationDatabase {
            medications: meds,
            archived_medications: Vec::new(),
        };
        // Save migrated data immediately
        save_database(&db);
        eprintln!("Migration complete!");
        return db;
    }

    // File is corrupted - neither format worked
    eprintln!("WARNING: Medications file is corrupted and cannot be parsed!");
    eprintln!("File location: {}", file_path.display());
    eprintln!("Creating backup at: {}.corrupted", file_path.display());

    // Create backup of corrupted file
    let backup_path = file_path.with_extension("json.corrupted");
    if let Err(backup_err) = fs::copy(&file_path, &backup_path) {
        eprintln!("Failed to create backup: {}", backup_err);
    } else {
        eprintln!("Backup created successfully.");
    }

    eprintln!("Starting with empty medication database.");
    MedicationDatabase {
        medications: Vec::new(),
        archived_medications: Vec::new(),
    }
}

/// Loads only the active medications from the database.
///
/// This is a convenience function for backwards compatibility with code
/// that only needs to work with active medications.
pub fn load_medications() -> Vec<Medication> {
    load_database().medications
}

/// Saves the complete medication database to disk atomically.
///
/// Uses atomic write pattern (write to temp file, then rename) to prevent
/// data corruption if interrupted. Sets file permissions to 0600 on Unix
/// systems for privacy.
pub fn save_database(db: &MedicationDatabase) {
    let file_path = get_data_file();

    let json = match serde_json::to_string_pretty(db) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Error: Failed to serialize medication database: {}", e);
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
        return;
    }

    // Set file permissions to 0600 (owner read/write only) for privacy
    // This is Unix-specific; Windows uses different permission models
    #[cfg(unix)]
    {
        if let Ok(metadata) = fs::metadata(&file_path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o600);
            if let Err(e) = fs::set_permissions(&file_path, perms) {
                eprintln!("Warning: Failed to set file permissions: {}", e);
            }
        }
    }
}

/// Saves active medications while preserving archived medications.
///
/// This is a convenience function that loads the full database, updates only
/// the active medications, and saves it back. Use this when you only want to
/// modify active medications without touching the archive.
pub fn save_medications(meds: &[Medication]) {
    let mut db = load_database();
    db.medications = meds.to_vec();
    save_database(&db);
}

/// Adds a new medication or unarchives an existing archived medication.
///
/// If a medication with the same name (case-insensitive) exists in the archive,
/// it will be moved back to active medications with updated fields but preserved
/// history. Otherwise, creates a new medication with empty history.
///
/// # Arguments
/// * `name` - Medication name
/// * `dose` - Dosage (e.g., "500mg", "10ml")
/// * `time` - Time to take (e.g., "8:00", "morning")
/// * `interval` - Frequency (e.g., "daily", "every 3 days")
/// * `notes` - Optional notes (e.g., "take with food")
///
/// # Validation
/// - Name, dose, and interval cannot be empty
/// - Time must be parseable by `time::parse_time`
/// - Name must not exist in active medications
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

    let mut db = load_database();
    let name_lower = name.to_lowercase();

    // Check if medication already exists in active list
    if db
        .medications
        .iter()
        .any(|m| m.name.to_lowercase() == name_lower)
    {
        eprintln!(
            "Error: Medication '{}' already exists in active medications!",
            name
        );
        return;
    }

    // Check if medication exists in archive - if so, unarchive it
    let archived_index = db
        .archived_medications
        .iter()
        .position(|m| m.name.to_lowercase() == name_lower);

    if let Some(index) = archived_index {
        // Unarchive: move from archived to active, updating fields
        let mut med = db.archived_medications.remove(index);

        // Update fields with new values, but preserve history
        med.dose = dose;
        med.time_of_day = time;
        med.medication_frequency = interval;
        med.notes = notes;
        med.taken = false;
        med.taken_at = String::new();
        // Keep last_dose_date and history

        db.medications.push(med.clone());
        save_database(&db);

        let history_count = med.history.len();
        println!("Unarchived medication: {}", name);
        if history_count > 0 {
            println!("  Restored {} dose record(s) from archive", history_count);
            println!("  View history with: pharm history {}", name);
        }
    } else {
        // Create fresh medication
        let med = Medication {
            name: name.clone(),
            dose,
            time_of_day: time,
            medication_frequency: interval,
            taken: false,
            taken_at: String::new(),
            last_dose_date: String::new(),
            notes,
            history: Vec::new(),
        };

        db.medications.push(med);
        save_database(&db);
        println!("Added medication: {}", name);
    }
}

/// Removes a medication from active list and archives it with full history.
///
/// The medication is moved to the archive, preserving all dose records. It can
/// be unarchived later by using `add_medication` with the same name.
///
/// # Safety
/// This function does NOT permanently delete medication data. All history is
/// preserved in the archive for medical compliance tracking.
pub fn remove_medication(name: String) {
    let mut db = load_database();
    let name_lower = name.to_lowercase();

    // Find and remove from active medications
    let mut found_med: Option<Medication> = None;
    db.medications.retain(|m| {
        if m.name.to_lowercase() == name_lower {
            found_med = Some(m.clone());
            false // Remove from active list
        } else {
            true // Keep in active list
        }
    });

    if let Some(med) = found_med {
        // Archive the medication with ALL its history
        db.archived_medications.push(med.clone());
        save_database(&db);

        let history_count = med.history.len();
        println!("Archived medication: {}", name);
        if history_count > 0 {
            println!("  Preserved {} dose record(s) in archive", history_count);
            println!(
                "  View history anytime with: pharm history {} --archived",
                name
            );
        }
    } else {
        println!("Medication '{}' not found!", name);
    }
}

pub fn list_medications(archived: bool, due: bool) {
    let db = load_database();

    let meds = if archived {
        &db.archived_medications
    } else {
        &db.medications
    };

    // Filter to due medications if requested
    let filtered_meds: Vec<&Medication> = if due {
        let now = chrono::Local::now();
        let today_date = now.date_naive();

        meds.iter()
            .filter(|med| {
                // Skip if already taken
                if med.taken {
                    return false;
                }

                // Check if time is due
                let time_is_due = crate::time::is_time_due(&med.time_of_day);
                if !time_is_due {
                    return false;
                }

                // Check if interval allows
                match crate::interval::parse_interval_to_days(&med.medication_frequency) {
                    Some(interval_days) => {
                        // Has interval - check if enough time has passed
                        if med.last_dose_date.is_empty() {
                            return true; // Never taken, so it's due
                        }

                        if let Ok(last_dose) =
                            chrono::NaiveDate::parse_from_str(&med.last_dose_date, "%Y-%m-%d")
                        {
                            let days_since_dose = (today_date - last_dose).num_days();
                            days_since_dose >= interval_days as i64
                        } else {
                            true // Can't parse, assume it's due
                        }
                    }
                    None => {
                        // PRN medication - skip from "due" list (no schedule)
                        false
                    }
                }
            })
            .collect()
    } else {
        meds.iter().collect()
    };

    if filtered_meds.is_empty() {
        if due {
            println!("No medications are currently due.");
        } else if archived {
            println!("No archived medications found.");
        } else {
            println!("No active medications found.");
        }
        return;
    }

    if due {
        println!("\nMedications Due Now:");
    } else if archived {
        println!("\nArchived Medications:");
    } else {
        println!("\nActive Medications:");
    }
    println!("{}", "=".repeat(60));

    for med in filtered_meds {
        println!("\n{}", med.name);
        println!("  Dose:     {}", med.dose);
        println!("  Time:     {}", med.time_of_day);
        println!("  Interval: {}", med.medication_frequency);

        if !archived {
            println!("  Taken:    {}", if med.taken { "✓" } else { "✗" });
            println!("  Taken At: {}", med.taken_at);
        }

        if let Some(notes) = &med.notes {
            println!("  Notes:    {}", notes);
        }

        if !med.history.is_empty() {
            println!("  History:  {} dose(s) recorded", med.history.len());
        }
    }
    println!();
}
/// Marks a medication as taken and records it in history.
///
/// Records the current timestamp and dose amount. Updates `last_dose_date`
/// for interval tracking. If the medication is archived, provides helpful
/// error message about how to unarchive it.
pub fn take_medication(name: String) {
    let mut db = load_database();
    let mut found = false;
    let name_lower = name.to_lowercase();
    let now = chrono::Local::now();
    let now_str = now.format("%H:%M:%S - %Y/%m/%d").to_string();
    let today = now.format("%Y-%m-%d").to_string();

    for med in db.medications.iter_mut() {
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

                    // Append to history
                    med.history.push(DoseRecord {
                        timestamp: now_str.clone(),
                        dose: med.dose.clone(),
                    });

                    found = true;
                    break;
                }
            }
        }
    }

    if found {
        save_database(&db);
        println!("Marked '{}' as taken at {}", name, now_str);
    } else {
        // Check if medication is archived
        let is_archived = db
            .archived_medications
            .iter()
            .any(|m| m.name.to_lowercase() == name_lower);

        if is_archived {
            eprintln!("Error: Medication '{}' is archived.", name);
            eprintln!(
                "To restart taking it, use: pharm add {} --dose <DOSE> --time <TIME> --freq <FREQ>",
                name
            );
        } else {
            eprintln!("Error: Medication '{}' not found!", name);
        }
    }
}
pub fn untake_medication(name: String) {
    let mut db = load_database();
    let mut found = false;
    let name_lower = name.to_lowercase();

    for med in db.medications.iter_mut() {
        if med.name.to_lowercase() == name_lower {
            if !med.taken {
                println!("Medication '{}' is not currently marked as taken", med.name);
                return;
            }
            med.taken = false;
            med.taken_at = String::new();
            // Keep last_dose_date - it's still needed for interval tracking

            // Remove last history entry (undo the dose)
            if !med.history.is_empty() {
                med.history.pop();
            }

            found = true;
            break;
        }
    }

    if found {
        save_database(&db);
        println!("Unmarked '{}' as taken", name);
    } else {
        // Check if medication is archived
        let is_archived = db
            .archived_medications
            .iter()
            .any(|m| m.name.to_lowercase() == name_lower);

        if is_archived {
            eprintln!("Error: Medication '{}' is archived.", name);
            eprintln!(
                "To restart taking it, use: pharm add {} --dose <DOSE> --time <TIME> --freq <FREQ>",
                name
            );
        } else {
            eprintln!("Error: Medication '{}' not found!", name);
        }
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

                // Append to history
                med.history.push(DoseRecord {
                    timestamp: now_str.clone(),
                    dose: med.dose.clone(),
                });
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
        let interval_days = match crate::interval::parse_interval_to_days(&med.medication_frequency)
        {
            Some(days) => days,
            None => continue, // Skip PRN (as-needed) medications - they don't reset on schedule
        };

        // Parse last dose date
        let should_reset = if med.last_dose_date.is_empty() {
            // No last dose date, reset to be safe
            true
        } else if let Ok(last_dose) =
            chrono::NaiveDate::parse_from_str(&med.last_dose_date, "%Y-%m-%d")
        {
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

/// Displays medication history with adherence metrics.
///
/// # Arguments
/// * `medication_name` - Optional specific medication name (shows all if None)
/// * `days` - Optional number of days to show (default: 30)
/// * `archived` - If true, only shows archived medications; if false, shows both active and archived
///
/// Shows:
/// - All dose records in reverse chronological order (newest first)
/// - Adherence percentage based on expected vs actual doses
/// - Whether medication is archived
pub fn display_history(medication_name: Option<String>, days: Option<u32>, archived: bool) {
    let db = load_database();

    // Combine active and archived medications based on flag
    let all_meds: Vec<&Medication> = if archived {
        // Show only archived
        db.archived_medications.iter().collect()
    } else {
        // Show both active and archived
        db.medications
            .iter()
            .chain(db.archived_medications.iter())
            .collect()
    };

    if all_meds.is_empty() {
        if archived {
            println!("No archived medications found.");
        } else {
            println!("No medications found.");
        }
        return;
    }

    let now = chrono::Local::now();
    let cutoff_date = days.map(|d| now - chrono::Duration::days(d as i64));

    // Filter medications if name provided
    let filtered_meds: Vec<&Medication> = if let Some(ref name) = medication_name {
        let name_lower = name.to_lowercase();
        all_meds
            .into_iter()
            .filter(|m| m.name.to_lowercase() == name_lower)
            .collect()
    } else {
        all_meds
    };

    if filtered_meds.is_empty() {
        if let Some(name) = medication_name {
            println!("Medication '{}' not found!", name);
        }
        return;
    }

    for med in filtered_meds {
        // Check if this medication is archived
        let is_archived = db.archived_medications.iter().any(|m| m.name == med.name);

        // Filter history by date if specified
        let history: Vec<&DoseRecord> = med
            .history
            .iter()
            .filter(|record| {
                if let Some(cutoff) = cutoff_date {
                    // Parse timestamp and compare
                    if let Ok(timestamp) = chrono::NaiveDateTime::parse_from_str(
                        &record.timestamp,
                        "%H:%M:%S - %Y/%m/%d",
                    ) {
                        let record_datetime = chrono::Local
                            .from_local_datetime(&timestamp)
                            .single()
                            .unwrap_or_else(chrono::Local::now);
                        record_datetime >= cutoff
                    } else {
                        true // Include if we can't parse
                    }
                } else {
                    true // No filter
                }
            })
            .collect();

        if history.is_empty() {
            if is_archived {
                println!("\n{} [ARCHIVED] - No history recorded", med.name);
            } else {
                println!("\n{} - No history recorded", med.name);
            }
            if days.is_some() {
                println!("  (No doses in last {} days)", days.unwrap());
            }
            continue;
        }

        if is_archived {
            println!("\n{} [ARCHIVED] - History", med.name);
        } else {
            println!("\n{} - History", med.name);
        }
        if let Some(d) = days {
            println!("  (Last {} days)", d);
        }
        println!("{}", "=".repeat(60));

        // Show history in reverse chronological order (newest first)
        for record in history.iter().rev() {
            println!("  {} - {}", record.timestamp, record.dose);
        }

        // Calculate adherence if we have a scheduled interval (not PRN)
        match crate::interval::parse_interval_to_days(&med.medication_frequency) {
            Some(interval_days) => {
                let days_to_check = days.unwrap_or(30);
                let expected_doses = (days_to_check / interval_days).max(1);
                let actual_doses = history.len() as u32;
                let adherence = if expected_doses > 0 {
                    (actual_doses as f32 / expected_doses as f32 * 100.0).min(100.0)
                } else {
                    0.0
                };

                println!(
                    "\n  Total doses: {} (Expected: ~{})",
                    actual_doses, expected_doses
                );
                println!("  Adherence: {:.1}%", adherence);
            }
            None => {
                // PRN medication - no adherence calculation
                println!("\n  Total doses: {} (as-needed)", history.len());
            }
        }
    }
    println!();
}
