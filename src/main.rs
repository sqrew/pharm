use clap::{Parser, Subcommand};

use daemon::run_daemon;
use database::{
    add_medication, edit_medication, list_medications, remove_medication, take_all_medications,
    take_medication, untake_medication,
};

pub mod daemon;
pub mod database;
pub mod interval;
pub mod time;

#[derive(Parser)]
#[command(name = "pharm")]
#[command(
    about = "CLI-first medication management tool",
    long_about = "A simple CLI tool to help remind you to take your medication. No data privacy is implemented."
)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
    Remove {
        /// Name of the medication
        name: String,
    },
    Take {
        name: String,
    },
    /// Mark a medication as NOT taken (undo)
    Untake {
        name: String,
    },
    TakeAll,
    /// Edit an existing medication
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
    List,
    /// Start the background daemon for reminders
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
        Commands::List => {
            list_medications();
        }
        Commands::Daemon => {
            run_daemon();
        }
    }
}
