# cpumon

[![CI](https://github.com/arghyadipchak/cpumon/actions/workflows/ci.yml/badge.svg?style=flat-square)](https://github.com/arghyadipchak/cpumon/actions)
[![Commitizen friendly](https://img.shields.io/badge/commitizen-friendly-brightgreen.svg?style=flat-square)](http://commitizen.github.io/cz-cli/)
[![Language: Rust](https://img.shields.io/badge/language-Rust-orange.svg?style=flat-square)](https://www.rust-lang.org/)
[![Platform: Linux](https://img.shields.io/badge/platform-Linux-lightgrey.svg?style=flat-square)](#-platform-support)
[![License: AGPL-3.0-or-later](https://img.shields.io/badge/license-AGPL--3.0--or--later-blue.svg?style=flat-square)](LICENSE)

A lightweight, real-time per-core CPU usage monitor

```text
TIME             AVG   CPU 0   CPU 1   CPU 2   CPU 3   CPU 4   CPU 5   CPU 6   CPU 7
22:05:00.200   14.2%   25.0%    8.5%   12.0%   18.2%    6.0%   15.5%   10.0%   18.4%
22:05:00.400   16.8%   30.2%   12.0%   10.5%   22.0%    8.5%   18.0%   14.2%   19.0%
```

[Features](#-features) •
[Installation](#-installation) •
[Usage](#-usage) •
[Output Formats](#-output-formats) •
[Timestamping](#-timestamp-formatting) •
[Shell Completions](#-shell-completions) •
[Platform Support](#-platform-support)

---

## ✨ Features

- **Per-Core Granularity**: Monitor individual cores (`0,2,4`), ranges (`0-3`), or all cores (`all`) with individual and aggregate CPU percentages
- **5 Native Output Formats**:
  - `human` — Aligned terminal table with ANSI color support
  - `csv` — Comma-separated values for spreadsheet analysis and plotting
  - `tsv` — Tab-separated values optimized for `awk`, `cut`, and Unix pipelines
  - `markdown` (or `md`) — GitHub-flavored markdown tables for reports and CI summaries
  - `json` — Line-delimited JSON objects for telemetry ingestion (`jq`, Fluentd, Vector)
- **Zero-Allocation Hot Path**: Pre-allocated internal buffers and direct output stream writing ensure zero heap allocations during measurement cycles
- **Drift-Free Scheduling**: Monotonic nanosecond timer prevents cumulative interval drift under heavy system load
- **Process Pinning (`--cpuset`)**: Pin `cpumon` itself to a specific core to eliminate monitoring interference on benchmarked cores
- **Versatile Timestamping**: Built-in presets (`time`, `datetime`, `iso`, `micros`, `millis`, `seconds`, `elapsed`) or arbitrary `strftime` patterns

---

## 📦 Installation

### Using Cargo (from Git)
```bash
cargo install --git https://github.com/arghyadipchak/cpumon.git
```

### Build from Source
```bash
git clone https://github.com/arghyadipchak/cpumon.git
cd cpumon
cargo build --release
```
The optimized release binary will be available at `target/release/cpumon`

---

## 🚀 Usage

### Basic Monitoring

```bash
# Monitor all CPU cores in real-time (200ms default interval)
cpumon

# Monitor specific cores or ranges
cpumon 0-3
cpumon 0,2,4,6

# Monitor only aggregate system average
cpumon -v avg

# Adjust sampling rate (e.g. 50ms for high-frequency profiling)
cpumon -i 50ms 0-3
```

### Logging & Benchmarking

```bash
# Record 1 minute of utilization to CSV with UTC ISO-8601 timestamps
cpumon -d 1m -i 1s -f csv -t iso > cpu_benchmark.csv

# Collect 100 fixed samples in JSON format
cpumon -n 100 -f json 0-7 > run_telemetry.jsonl

# Record strictly per-core metrics to CSV without aggregate average
cpumon -d 30s -i 250ms -v cores -f csv 0-3 > per_core.csv
```

### Common Recipes

#### 1. Zero-Interference Benchmark Profiling
Pin `cpumon` itself to core 0 with `-c 0` to monitor worker cores 1–7 without monitor polling overhead skewing benchmark results:
```bash
cpumon -c 0 -i 50ms 1-7 -f csv -t elapsed > benchmark.csv
```

#### 2. Real-Time Alerting with `jq`
Stream JSON metrics and filter samples in real time where aggregate CPU load exceeds 80%:
```bash
cpumon -v avg -f json | jq --unbuffered -c 'select(.avg > 80.0)'
```

#### 3. Unix Pipelines with `awk`
Extract live timestamp and Core 0 usage for downstream processing scripts using TSV:
```bash
cpumon -f tsv --no-header 0 | awk '{print "Core 0:", $3 "%", "at", $1}'
```

#### 4. GitHub Actions CI Step Summary
Record CPU metrics during a benchmark run and render a Markdown table directly into GitHub Actions summary:
```bash
cpumon -n 10 -f md -t elapsed 0-3 >> $GITHUB_STEP_SUMMARY
```

---

## 📋 Command Reference

```text
Usage: cpumon [OPTIONS] [CORES]

Arguments:
  [CORES]  CPU cores to monitor (e.g. '0', '0,2,4', '0-3', 'all') [default: all]

Options:
  -c, --cpuset <CORE_ID>           Pin the monitoring process to a specific CPU core ID
  -i, --interval <DURATION>        Sampling interval (e.g. '200ms', '1s') [default: 200ms]
  -n, --count <COUNT>              Number of samples to collect before exiting
  -d, --duration <DURATION>        Total duration to monitor before exiting (e.g. '10s', '1m')
  -f, --format <FORMAT>            Output format [default: human] [possible values: human, csv, tsv, json, markdown]
  -t, --time-format <TIME_FORMAT>  Timestamp format preset or custom strftime pattern [default: auto]
      --no-header                  Suppress table or column headers
  -v, --view <MODE>                Metrics view mode [default: both] [possible values: both, cores, avg]
  -h, --help                       Print help (see more with '--help')
  -V, --version                    Print version
```

---

## 📊 Output Formats

### 1. Human Table (`-f human`)
```text
TIME             AVG   CPU 0   CPU 1   CPU 2   CPU 3
21:58:55.350   12.8%   20.0%    5.6%   15.0%   10.5%
21:58:55.550   17.1%    5.6%   28.6%   14.3%   20.0%
```

### 2. CSV (`-f csv`)
```csv
timestamp,avg,core_0,core_1
1787156557629638,2.50,5.00,0.00
1787156557829649,7.32,5.56,9.09
```

### 3. TSV (`-f tsv`)
```tsv
timestamp	avg	core_0	core_1
1787156558074617	5.26	10.53	0.00
1787156558274360	5.26	10.53	0.00
```

### 4. Markdown (`-f md` / `-f markdown`)
```markdown
| TIME | AVG | CPU 0 | CPU 1 |
| :--- | :---: | :---: | :---: |
| 21:52:38.518 |   7.5% |  15.0% |   0.0% |
| 21:52:38.718 |   7.5% |  15.0% |   0.0% |
```

### 5. Line-Delimited JSON (`-f json`)
```json
{"timestamp":1787156558962378,"avg":5.56,"cores":{"0":11.11,"1":0.00}}
{"timestamp":1787156559162581,"avg":7.50,"cores":{"0":15.00,"1":0.00}}
```

---

## ⏰ Timestamp Formatting

| Preset | Description | Output Example |
| :--- | :--- | :--- |
| `auto` *(default)* | Clock time (`human`/`md`) or Epoch micros (`csv`/`tsv`/`json`) | `21:58:55.350` / `1787156557629638` |
| `time` | Local time with milliseconds (`%H:%M:%S%.3f`) | `21:58:55.350` |
| `datetime` | Local date and time (`%Y-%m-%d %H:%M:%S`) | `2026-08-19 21:58:55` |
| `iso` | UTC ISO-8601 (`%Y-%m-%dT%H:%M:%S%.3fZ`) | `2026-08-19T16:28:55.350Z` |
| `elapsed` | Elapsed time from start (`HH:MM:SS.mmm`) | `00:01:23.456` |
| `micros` | Unix epoch microseconds (numeric) | `1787156557629638` |
| `millis` | Unix epoch milliseconds (numeric) | `1787156557629` |
| `seconds` | Unix epoch seconds (numeric) | `1787156557` |
| *Custom* | Any valid `strftime` pattern (e.g. `"%Y/%m/%d %H:%M"`) | `2026/08/19 21:58` |

---

## 🐚 Shell Completions

Static shell completion scripts are automatically generated at build time into the `completions/` directory:

| Shell | Installation |
| :--- | :--- |
| **Bash** | `source completions/cpumon.bash` or copy to `/etc/bash_completion.d/` |
| **Zsh** | Copy `completions/_cpumon` to a directory in your `$fpath` |
| **Fish** | `cp completions/cpumon.fish ~/.config/fish/completions/` |

---

## 🐧 Platform Support

> [!NOTE]
> `cpumon` is developed and optimized primarily for **Linux**. While it may compile on other platforms (such as macOS or Windows), non-Linux operating systems are neither actively tested nor officially supported.

---

## 📄 License

This project is licensed under the **GNU Affero General Public License v3.0 or later** ([AGPL-3.0-or-later](LICENSE))
