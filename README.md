# pharm

A CLI-first medication management and reminder tool for the terminal.

**pharm** helps you track medications, log doses, and receive timely reminders through desktop notifications. Built in Rust for speed, safety, and reliability.

## Features

- **Track all medications** - pills, liquids, injections, inhalers, patches
- **Smart reminders** - background daemon with desktop notifications
- **Flexible scheduling** - daily, weekly, custom intervals (e.g., "every 3 days")
- **Interval tracking** - prevents accidental overdose by respecting medication frequency
- **Medication history** - track all doses taken with timestamps and adherence metrics
- **Archive system** - removed medications preserve complete history for medical records
- **Flexible time parsing** - use "8:00", "morning", "evening", or "bedtime"
- **Notes support** - add reminders like "take with food"
- **Command aliases** - faster typing with short commands (e.g., `pharm t` for take)
- **Local storage** - your health data stays on your machine (`~/.pharm.json`)
- **Privacy-focused** - file permissions set to 0600 (Unix) for medical data protection
- **Simple workflow** - add, list, take, edit medications with ease

## Installation

### From source

```bash
git clone https://github.com/sqrew/pharm
cd pharm
cargo install --path .
```

### From crates.io (once published)

```bash
cargo install pharm
```

Verify installation:

```bash
pharm --version
```

## Quick Start

```bash
# Add a scheduled medication
pharm add "Aspirin" --dose 500mg --time 8:00 --freq daily

# Add with notes
pharm add "Insulin" --dose 10u --time morning --freq daily --notes "Take with breakfast"

# Add as-needed medication
pharm add "Ibuprofen" --dose 200mg --time prn --freq prn --notes "For headaches"

# List all medications
pharm list

# List only medications due right now
pharm list --due

# Mark a dose as taken
pharm take "Aspirin"

# Start the reminder daemon
pharm daemon
```

## Usage

### Adding Medications

```bash
pharm add <NAME> --dose <DOSE> --time <TIME> --freq <FREQUENCY> [--notes <NOTES>]
```

**Examples:**

```bash
# Standard format
pharm add "Lisinopril" -d 10mg -t 08:00 -f daily

# Named times (morning, noon, afternoon, evening, bedtime, midnight)
pharm add "Vitamin D" -d 1000IU -t morning -f daily

# Custom intervals
pharm add "B12 Shot" -d 1mg -t "9:00" -f "every 7 days"
pharm add "Allergy Med" -d 10mg -t evening -f "every 3 days"

# With notes
pharm add "Metformin" -d 500mg -t dinner -f "twice daily" -n "Take with food"

# PRN (as-needed) medications
pharm add "Tylenol" -d 500mg -t prn -f prn -n "For pain"
pharm add "Benadryl" -d 25mg -t prn -f "as needed" -n "For allergies"
```

**Supported time formats:**
- `8:00`, `08:30`, `14:15` (HH:MM format)
- `8`, `14` (hour only, assumes :00)
- Named: `morning` (8am), `noon` (12pm), `evening` (6pm), `bedtime` (9pm)

**Supported frequencies:**
- `daily`, `weekly`, `monthly`
- `every X days` (e.g., `every 3 days`)
- `every X weeks` (e.g., `every 2 weeks`)
- `twice daily`, `3 times daily` (treated as daily)
- `prn`, `as needed` (as-needed medications with no schedule)

### Listing Medications

```bash
# List all active medications
pharm list

# List archived medications
pharm list --archived

# List only medications due right now
pharm list --due
```

The `--due` flag shows only untaken medications that are currently due (past their scheduled time and interval). Perfect for answering "What do I need to take right now?"

### Taking Medications

```bash
# Mark a single medication as taken
pharm take "Aspirin"

# Mark all medications as taken (for current interval)
pharm take-all

# Undo accidental marking
pharm untake "Aspirin"
```

### Editing Medications

```bash
# Edit any field(s)
pharm edit "Aspirin" --dose 1000mg
pharm edit "Aspirin" --time evening --freq weekly
pharm edit "Aspirin" --notes "New instructions"

# Clear notes
pharm edit "Aspirin" --notes ""
```

### Removing Medications

```bash
pharm remove "Aspirin"
```

When you remove a medication, it's **archived** with its complete history preserved. This means:
- All dose records are kept for medical compliance tracking
- You can view the history anytime with `pharm history <name>`
- To restart taking it, just use `pharm add` again - it will unarchive automatically

### Running the Daemon

The daemon monitors your medications and sends desktop notifications when doses are due.

```bash
# Start daemon (runs in foreground)
pharm daemon

# Run in background
nohup pharm daemon > ~/pharm.log 2>&1 &

# Check daemon logs
tail -f ~/pharm.log
```

**Daemon features:**
- Checks every 60 seconds for due medications
- Only notifies once per medication per daemon session (resets on daemon restart or midnight)
- Respects medication intervals (won't remind for weekly meds every day)
- Resets medications at midnight and on daemon startup (ensures correct state even if daemon was off overnight)
- Desktop notifications persist until dismissed

### Interval Safety

**pharm** tracks the last time each medication was taken and respects the specified frequency to prevent accidental overdose:

- **Daily medications** reset at midnight each day
- **Weekly medications** only remind after 7 days have passed
- **Custom intervals** (e.g., "every 3 days") track the exact number of days

This means if you take a weekly medication on Monday, you won't get reminders again until the following Monday, even if the daemon restarts.

### PRN (As-Needed) Medications

For medications taken only when needed (e.g., pain relievers, allergy meds):

```bash
# Add PRN medication
pharm add "Ibuprofen" --dose 200mg --time prn --freq prn --notes "For headaches"

# Take it whenever needed
pharm take "Ibuprofen"

# View history
pharm history "Ibuprofen"
```

**PRN medications:**
- Have no scheduled reminders (daemon skips them)
- Can be taken anytime without interval restrictions
- Still track complete history with timestamps
- Show "as-needed" instead of adherence percentage in history
- Won't appear in `pharm list --due` (no schedule)

**Supported PRN markers:** `prn`, `as needed`, `as-needed`, `when needed`

### Viewing History

Track your medication adherence over time:

```bash
# View all medication history (last 30 days by default)
pharm history

# View history for specific medication
pharm history "Aspirin"

# View last 7 days
pharm history --days 7

# View longer period
pharm history "Aspirin" --days 90

# View only archived medications
pharm history --archived
```

History includes:
- Complete timestamp for every dose taken
- Dose amount at time of taking (in case it changed)
- Adherence percentage based on expected vs actual doses
- Works for both active and archived medications

### Archived Medications

View or manage archived medications:

```bash
# List archived medications
pharm list --archived

# View archived medication history
pharm history "Old Med" --archived

# Restart an archived medication (unarchive)
pharm add "Old Med" --dose 10mg --time morning --freq daily
```

When you unarchive a medication by adding it again:
- All historical dose records are preserved
- Fields (dose, time, frequency) are updated to new values
- Medication moves back to active list

### Data Storage

All medication data is stored in `~/.pharm.json` as human-readable JSON. You can:
- Back it up: `cp ~/.pharm.json ~/.pharm.json.backup`
- View it: `cat ~/.pharm.json`
- Edit it manually (if needed): `nano ~/.pharm.json`

The file contains:
- `medications`: Active medications you're currently taking
- `archived_medications`: Removed medications with complete history preserved
- Each medication includes full dose history with timestamps

File permissions are automatically set to **0600** (owner read/write only) on Unix systems for medical data privacy.

### Notification System

Uses your desktop environment's native notification system:
- **Linux**: D-Bus notifications
- **macOS**: Notification Center
- **Windows**: Windows notification system

## Safety Considerations

⚠️ **Important:** This tool is designed to *assist* with medication management, not replace medical advice or professional healthcare.

- Always consult your doctor or pharmacist about medication schedules
- Use this tool as a reminder aid, not as medical guidance
- Verify medication information with healthcare professionals
- The interval tracking helps prevent accidental double-dosing, but you are responsible for taking medications correctly

## Commands Reference

| Command | Alias | Description |
|---------|-------|-------------|
| `pharm add` | `a`, `ad` | Add a new medication (or unarchive if exists) |
| `pharm list` | `l` | List active medications |
| `pharm list --archived` | `l -a` | List archived medications |
| `pharm list --due` | `l --due` | List only medications due right now |
| `pharm take <name>` | `t` | Mark medication as taken |
| `pharm untake <name>` | `u` | Undo marking as taken |
| `pharm take-all` | `ta` | Mark all medications as taken |
| `pharm edit <name>` | `e` | Edit medication details |
| `pharm remove <name>` | `r` | Remove (archive) a medication |
| `pharm history` | `h` | View medication history |
| `pharm history <name>` | `h <name>` | View specific medication history |
| `pharm history --days 7` | `h -d 7` | View last 7 days of history |
| `pharm daemon` | `d` | Start reminder daemon |
| `pharm --help` | | Show help |
| `pharm --version` | | Show version |

### Command Aliases

All commands support short aliases for faster typing:

```bash
# These are equivalent:
pharm add "Aspirin" -d 500mg -t morning -f daily
pharm a "Aspirin" -d 500mg -t morning -f daily

# More examples:
pharm t "Aspirin"      # Take
pharm l                # List
pharm h --days 7       # History
pharm r "Old Med"      # Remove
```

## Building from Source

```bash
git clone https://github.com/yourusername/pharm
cd pharm
cargo build --release
./target/release/pharm --version
```

## Running Tests

```bash
cargo test
```

## Contributing

Contributions are welcome! This is a simple, focused tool that does one thing well. When contributing, please:

- Keep the code simple and readable
- Add tests for new functionality
- Update documentation

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

Built with:
- [rust](https://rust-lang.org/) - Programming Language
Using the following crates:
- [clap](https://github.com/clap-rs/clap) - Command line argument parsing
- [serde](https://serde.rs/) - Serialization framework
- [chrono](https://github.com/chronotope/chrono) - Date and time library
- [notify-rust](https://github.com/hoodie/notify-rust) - Desktop notifications

---

**pharm** - Your terminal pharmacist helper 💊
