# pharm

A CLI-first medication management and reminder tool for the terminal.

**pharm** helps you track medications, log doses, and receive timely reminders through desktop notifications. Built in Rust for speed, safety, and reliability.

## Features

- 📋 **Track all medications** - pills, liquids, injections, inhalers, patches
- ⏰ **Smart reminders** - background daemon with desktop notifications
- 🔄 **Flexible scheduling** - daily, weekly, custom intervals (e.g., "every 3 days")
- 📅 **Interval tracking** - prevents accidental overdose by respecting medication frequency
- ⏱️ **Flexible time parsing** - use "8:00", "morning", "evening", or "bedtime"
- 📝 **Notes support** - add reminders like "take with food"
- 🔒 **Local storage** - your health data stays on your machine (`~/.pharm.json`)
- ✅ **Simple workflow** - add, list, take, edit medications with ease

## Installation

### From source

```bash
git clone https://github.com/yourusername/pharm
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
# Add a medication
pharm add "Aspirin" --dose 500mg --time 8:00 --freq daily

# Add with notes
pharm add "Insulin" --dose 10u --time morning --freq daily --notes "Take with breakfast"

# List all medications
pharm list

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

### Listing Medications

```bash
pharm list
```

Shows all medications with their dose, time, interval, taken status, and notes.

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
- Only notifies once per medication per day
- Respects medication intervals (won't remind for weekly meds every day)
- Resets daily medications at midnight
- Desktop notifications persist until dismissed

### Auto-start on Login (systemd)

Create `~/.config/systemd/user/pharm.service`:

```ini
[Unit]
Description=Pharm medication reminder daemon
After=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.cargo/bin/pharm daemon
Restart=on-failure
StandardOutput=append:%h/pharm.log
StandardError=append:%h/pharm.log

[Install]
WantedBy=default.target
```

Enable and start:

```bash
systemctl --user enable pharm
systemctl --user start pharm
systemctl --user status pharm
```

View logs:

```bash
journalctl --user -u pharm -f
```

## How It Works

### Interval Safety

**pharm** tracks the last time each medication was taken and respects the specified frequency to prevent accidental overdose:

- **Daily medications** reset at midnight each day
- **Weekly medications** only remind after 7 days have passed
- **Custom intervals** (e.g., "every 3 days") track the exact number of days

This means if you take a weekly medication on Monday, you won't get reminders again until the following Monday, even if the daemon restarts.

### Data Storage

All medication data is stored in `~/.pharm.json` as human-readable JSON. You can:
- Back it up: `cp ~/.pharm.json ~/.pharm.json.backup`
- View it: `cat ~/.pharm.json`
- Edit it manually (if needed): `nano ~/.pharm.json`

The file is created with your default umask permissions.

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

| Command | Description |
|---------|-------------|
| `pharm add` | Add a new medication |
| `pharm list` | List all medications |
| `pharm take <name>` | Mark medication as taken |
| `pharm untake <name>` | Undo marking as taken |
| `pharm take-all` | Mark all medications as taken |
| `pharm edit <name>` | Edit medication details |
| `pharm remove <name>` | Remove a medication |
| `pharm daemon` | Start reminder daemon |
| `pharm --help` | Show help |
| `pharm --version` | Show version |

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
- Follow existing code style
- Update documentation

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

Built with:
- [clap](https://github.com/clap-rs/clap) - Command line argument parsing
- [serde](https://serde.rs/) - Serialization framework
- [chrono](https://github.com/chronotope/chrono) - Date and time library
- [notify-rust](https://github.com/hoodie/notify-rust) - Desktop notifications

---

**pharm** - Your terminal pharmacist 💊
