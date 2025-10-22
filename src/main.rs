use clap::{Parser, Subcommand};

use daemon::run_daemon;
use database::{
    add_medication, display_history, edit_medication, list_medications, remove_medication,
    take_all_medications, take_medication, untake_medication,
};

pub mod daemon;
pub mod database;
pub mod interval;
pub mod time;

#[derive(Parser)]
#[command(name = "pharm")]
#[command(
    about = "CLI-first medication management tool",
    long_about = "A simple CLI tool to help remind you to take your medication and maintain medication compliance. No data privacy is implemented. Everything is saved as JSON for easy import/export."
)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(visible_aliases = ["a", "ad"])]
    /// Add a new medication
    Add {
        /// Name of the medication
        name: String,
        /// Dosage (e.g., "500mg", "10ml")
        #[arg(short, long)]
        dose: String,
        /// Time to take (e.g., "8:00", "08:30", "8" or "morning", "noon", "evening")
        #[arg(short, long)]
        time: String,
        /// How often (e.g., "daily", "twice daily", "every 8 hours")
        #[arg(short, long)]
        freq: String,
        /// Optional notes
        #[arg(short, long)]
        notes: Option<String>,
    },
    /// Remove a medication
    #[command(visible_alias = "r")]
    Remove {
        /// Name of the medication
        name: String,
    },
    /// Mark a medication as taken
    #[command(visible_alias = "t")]
    Take { name: String },
    #[command(visible_alias = "u")]
    /// Mark a medication as NOT taken (undo)
    Untake { name: String },
    /// Mark ALL medications as taken
    #[command(visible_alias = "ta")]
    TakeAll,
    /// Edit an existing medication
    #[command(visible_alias = "e")]
    Edit {
        /// Name of the medication to edit
        name: String,
        /// New dosage
        #[arg(long)]
        dose: Option<String>,
        /// New time to take
        #[arg(long)]
        time: Option<String>,
        /// New frequency
        #[arg(long)]
        freq: Option<String>,
        /// New notes (use empty string to clear)
        #[arg(long)]
        notes: Option<String>,
    },
    /// List all medications
    #[command(visible_aliases = ["l", "s", "show"])]
    List {
        /// Show archived medications instead of active ones
        #[arg(short, long)]
        archived: bool,
        /// Show only medications that are due now (past scheduled time and interval)
        #[arg(long)]
        due: bool,
    },
    /// View medication history
    #[command(visible_alias = "h")]
    History {
        /// Name of medication (optional - shows all if not specified)
        name: Option<String>,
        /// Number of days to show (default: 30)
        #[arg(short, long)]
        days: Option<u32>,
        /// Show only archived medications
        #[arg(short, long)]
        archived: bool,
    },
    /// Start the background daemon for reminders
    #[command(visible_alias = "d")]
    Daemon,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add {
            name,
            dose,
            time,
            freq,
            notes,
        } => {
            add_medication(name, dose, time, freq, notes);
        }
        Commands::Remove { name } => {
            remove_medication(name);
        }
        Commands::Take { name } => {
            take_medication(name);
        }
        Commands::Untake { name } => {
            untake_medication(name);
        }
        Commands::TakeAll => take_all_medications(),
        Commands::Edit {
            name,
            dose,
            time,
            freq,
            notes,
        } => {
            edit_medication(name, dose, time, freq, notes);
        }
        Commands::List { archived, due } => {
            list_medications(archived, due);
        }
        Commands::History {
            name,
            days,
            archived,
        } => {
            display_history(name, days, archived);
        }
        Commands::Daemon => {
            run_daemon();
        }
    }
}
