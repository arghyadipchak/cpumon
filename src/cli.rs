use std::{fmt, str};

use clap::{
    Parser, ValueEnum,
    builder::styling::{AnsiColor, Effects, Styles},
};
use duration_string::DurationString;
use sysinfo::MINIMUM_CPU_UPDATE_INTERVAL;

use crate::utils;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CliError {
    #[error("no CPU cores found")]
    NoCoresFound,

    #[error("invalid core specification '{0}' (expected 'all', index, or '0-3')")]
    InvalidCpuSpec(String),

    #[error("empty core specification in list (unexpected comma)")]
    EmptySpec,

    #[error("invalid core range '{start}-{end}': start ({start}) > end ({end})")]
    ReversedRange { start: usize, end: usize },

    #[error(
        "core index {index} out of bounds (available: 0..{max})",
        max = total.saturating_sub(1)
    )]
    IndexOutOfBounds { index: usize, total: usize },

    #[error(
        "core range {start}-{end} out of bounds (available: 0..{max})",
        max = total.saturating_sub(1)
    )]
    RangeOutOfBounds {
        start: usize,
        end: usize,
        total: usize,
    },
}

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default());

/// A lightweight, real-time per-core CPU usage monitor
#[derive(Parser, Debug)]
#[command(
  version,
  styles = STYLES,
  about = "A lightweight, real-time per-core CPU usage monitor",
  after_help = "EXAMPLES:\n  cpumon                           Monitor all CPU cores\n  cpumon 0-3                       Monitor cores 0, 1, 2, and 3\n  cpumon -v avg                    Monitor overall aggregate average only\n  cpumon -v cores -f csv 0,1       Record per-core CSV without average column\n  cpumon -i 500ms -d 10s           Sample every 500ms for 10 seconds\n  cpumon -n 5 -f json 0,1          Output 5 samples in JSON\n  cpumon -f csv -t iso > out.csv   Record to CSV with ISO-8601 timestamps"
)]
pub struct Cli {
    /// Pin the monitoring process to a specific CPU core ID.
    #[arg(
        short,
        long,
        value_name = "CORE_ID",
        help = "Pin the monitoring process to a specific CPU core ID"
    )]
    pub cpuset: Option<usize>,

    /// CPU cores to monitor (e.g. '0', '0,2,4', '0-3', or 'all').
    #[arg(
        default_value = "all",
        value_name = "CORES",
        help = "CPU cores to monitor (e.g. '0', '0,2,4', '0-3', 'all')",
        long_help = "CPU cores to monitor. Accepts individual IDs, comma-separated lists, inclusive ranges, or 'all'.\nExamples: '0', '0,2,4', '0-3', 'all'"
    )]
    pub cpu_range: CpuRange,

    /// Sampling interval (e.g. '200ms', '1s').
    #[arg(
    short,
    long,
    value_name = "DURATION",
    default_value_t = DurationString::from(MINIMUM_CPU_UPDATE_INTERVAL),
    help = "Sampling interval (e.g. '200ms', '1s')",
    long_help = "Sampling interval.\nSupported units: 'ms' (milliseconds), 's' (seconds), 'm' (minutes), 'h' (hours).\nExamples: '100ms', '500ms', '1.5s', '10s'"
  )]
    pub interval: DurationString,

    /// Number of samples to collect before exiting.
    #[arg(
        short = 'n',
        long,
        value_name = "COUNT",
        help = "Number of samples to collect before exiting"
    )]
    pub count: Option<usize>,

    /// Total duration to monitor before exiting (e.g. '10s', '1m', '30s').
    #[arg(
        short = 'd',
        long,
        value_name = "DURATION",
        help = "Total duration to monitor before exiting (e.g. '10s', '1m')",
        long_help = "Total duration to monitor before exiting.\nSupported units: 'ms' (milliseconds), 's' (seconds), 'm' (minutes), 'h' (hours).\nExamples: '10s', '30s', '1m', '5m'"
    )]
    pub duration: Option<DurationString>,

    /// Output format.
    #[arg(
        short,
        long,
        value_name = "FORMAT",
        default_value = "human",
        help = "Output format"
    )]
    pub format: Format,

    /// Timestamp format preset or custom strftime pattern.
    #[arg(
    short = 't',
    long = "time-format",
    value_name = "TIME_FORMAT",
    default_value_t = TimeFormat::Auto,
    help = "Timestamp format preset or custom strftime pattern (e.g. '%Y-%m-%d %H:%M:%S')",
    long_help = "Timestamp format preset or custom strftime pattern.\n\nPRESETS:\n  auto      Context-aware default (clock time for human/markdown, epoch for CSV/TSV/JSON)\n  time      Local time with milliseconds: HH:MM:SS.mmm (alias: hms)\n  datetime  Local date and time: YYYY-MM-DD HH:MM:SS (alias: local)\n  iso       UTC ISO-8601: YYYY-MM-DDTHH:MM:SS.mmmZ (alias: rfc3339)\n  elapsed   Elapsed time from start: HH:MM:SS.mmm (alias: rel)\n  seconds   Unix epoch seconds (numeric) (alias: s, sec)\n  millis    Unix epoch milliseconds (numeric) (alias: ms)\n  micros    Unix epoch microseconds (numeric) (alias: us)\n\nCUSTOM PATTERNS:\n  Any standard strftime pattern, e.g. '%Y-%m-%d %H:%M:%S', '%s', '%T'"
  )]
    pub time_format: TimeFormat,

    /// Suppress table or column headers (e.g. in CSV format).
    #[arg(
        long,
        default_value_t = false,
        help = "Suppress table or column headers"
    )]
    pub no_header: bool,

    /// Metrics view mode (both, cores, avg).
    #[arg(
        short = 'v',
        long = "view",
        value_name = "MODE",
        default_value = "both",
        help = "Metrics view mode (both, cores, avg)"
    )]
    pub view: ViewMode,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum ViewMode {
    /// Show both aggregate average and per-core metrics (default).
    #[default]
    #[value(alias = "all", alias = "full")]
    Both,

    /// Show only individual core metrics without aggregate average.
    #[value(alias = "per-core", alias = "no-avg")]
    Cores,

    /// Show only aggregate average metric without individual cores.
    #[value(alias = "average", alias = "only-avg")]
    Avg,
}

impl fmt::Display for ViewMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Both => write!(f, "both"),
            Self::Cores => write!(f, "cores"),
            Self::Avg => write!(f, "avg"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CpuRange(pub Vec<usize>);

impl CpuRange {
    #[must_use]
    pub fn new(mut cpus: Vec<usize>) -> Self {
        cpus.sort_unstable();
        cpus.dedup();
        Self(cpus)
    }

    #[cfg(test)]
    #[must_use]
    pub fn cores(&self) -> &[usize] {
        &self.0
    }

    #[must_use]
    pub fn into_inner(self) -> Vec<usize> {
        self.0
    }
}

impl str::FromStr for CpuRange {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Ok(cpu_count) = utils::get_cpu_count() else {
            return Err(Self::Err::NoCoresFound);
        };

        let s = s.trim();
        if s.is_empty() || s.eq_ignore_ascii_case("all") {
            return Ok(Self((0..cpu_count).collect()));
        }

        let mut cpus = Vec::new();
        for part in s.split(',') {
            let part = part.trim();
            if part.is_empty() {
                return Err(Self::Err::EmptySpec);
            }

            if let Some((start_str, end_str)) = part.split_once('-') {
                let start = start_str
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| Self::Err::InvalidCpuSpec(part.to_string()))?;
                let end = end_str
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| Self::Err::InvalidCpuSpec(part.to_string()))?;

                if start > end {
                    return Err(Self::Err::ReversedRange { start, end });
                }
                if start >= cpu_count || end >= cpu_count {
                    return Err(Self::Err::RangeOutOfBounds {
                        start,
                        end,
                        total: cpu_count,
                    });
                }
                cpus.extend(start..=end);
            } else if let Ok(idx) = part.parse::<usize>() {
                if idx >= cpu_count {
                    return Err(Self::Err::IndexOutOfBounds {
                        index: idx,
                        total: cpu_count,
                    });
                }
                cpus.push(idx);
            } else {
                return Err(Self::Err::InvalidCpuSpec(part.to_string()));
            }
        }

        if cpus.is_empty() {
            return Err(Self::Err::NoCoresFound);
        }

        Ok(Self::new(cpus))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Format {
    #[default]
    Human,
    Csv,
    Tsv,
    Json,
    #[value(alias = "md")]
    Markdown,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Human => write!(f, "human"),
            Self::Csv => write!(f, "csv"),
            Self::Tsv => write!(f, "tsv"),
            Self::Json => write!(f, "json"),
            Self::Markdown => write!(f, "markdown"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum TimeFormat {
    #[default]
    Auto,
    Time,
    DateTime,
    Micros,
    Millis,
    Seconds,
    Iso,
    Elapsed,
    Custom(String),
}

impl TimeFormat {
    #[must_use]
    pub fn resolve(&self, format: Format) -> Self {
        match self {
            Self::Auto => match format {
                Format::Human | Format::Markdown => Self::Time,
                Format::Csv | Format::Tsv | Format::Json => Self::Micros,
            },
            explicit => explicit.clone(),
        }
    }
}

impl str::FromStr for TimeFormat {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.eq_ignore_ascii_case("auto") {
            Ok(Self::Auto)
        } else if s.eq_ignore_ascii_case("time") || s.eq_ignore_ascii_case("hms") {
            Ok(Self::Time)
        } else if s.eq_ignore_ascii_case("datetime")
            || s.eq_ignore_ascii_case("date-time")
            || s.eq_ignore_ascii_case("local")
        {
            Ok(Self::DateTime)
        } else if s.eq_ignore_ascii_case("micros") || s.eq_ignore_ascii_case("us") {
            Ok(Self::Micros)
        } else if s.eq_ignore_ascii_case("millis") || s.eq_ignore_ascii_case("ms") {
            Ok(Self::Millis)
        } else if s.eq_ignore_ascii_case("seconds")
            || s.eq_ignore_ascii_case("s")
            || s.eq_ignore_ascii_case("sec")
        {
            Ok(Self::Seconds)
        } else if s.eq_ignore_ascii_case("iso") || s.eq_ignore_ascii_case("rfc3339") {
            Ok(Self::Iso)
        } else if s.eq_ignore_ascii_case("elapsed") || s.eq_ignore_ascii_case("rel") {
            Ok(Self::Elapsed)
        } else {
            Ok(Self::Custom(s.to_string()))
        }
    }
}

impl fmt::Display for TimeFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Time => write!(f, "time"),
            Self::DateTime => write!(f, "datetime"),
            Self::Micros => write!(f, "micros"),
            Self::Millis => write!(f, "millis"),
            Self::Seconds => write!(f, "seconds"),
            Self::Iso => write!(f, "iso"),
            Self::Elapsed => write!(f, "elapsed"),
            Self::Custom(custom) => write!(f, "{custom}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_range_dedup_and_sort() {
        let range = CpuRange::new(vec![3, 1, 2, 1, 3]);
        assert_eq!(range.cores(), &[1, 2, 3]);
    }

    #[test]
    fn test_cpu_range_from_str() {
        let cpu_count = utils::get_cpu_count().unwrap();
        if cpu_count >= 2 {
            let range: CpuRange = "0,1".parse().unwrap();
            assert_eq!(range.cores(), &[0, 1]);

            let range: CpuRange = "0-1".parse().unwrap();
            assert_eq!(range.cores(), &[0, 1]);

            let all_range: CpuRange = "all".parse().unwrap();
            assert_eq!(all_range.cores().len(), cpu_count);
        }
    }

    #[test]
    fn test_cpu_range_invalid() {
        assert_eq!(
            "foo".parse::<CpuRange>().unwrap_err(),
            CliError::InvalidCpuSpec("foo".to_string())
        );
        assert_eq!(
            "0-".parse::<CpuRange>().unwrap_err(),
            CliError::InvalidCpuSpec("0-".to_string())
        );
        assert_eq!(
            "-1".parse::<CpuRange>().unwrap_err(),
            CliError::InvalidCpuSpec("-1".to_string())
        );
        assert_eq!(
            "3-1".parse::<CpuRange>().unwrap_err(),
            CliError::ReversedRange { start: 3, end: 1 }
        );
        assert_eq!("0,,1".parse::<CpuRange>().unwrap_err(), CliError::EmptySpec);
    }

    #[test]
    fn test_format_display() {
        assert_eq!(Format::Human.to_string(), "human");
        assert_eq!(Format::Csv.to_string(), "csv");
        assert_eq!(Format::Tsv.to_string(), "tsv");
        assert_eq!(Format::Json.to_string(), "json");
        assert_eq!(Format::Markdown.to_string(), "markdown");
    }

    #[test]
    fn test_time_format_resolve_and_display() {
        assert_eq!(TimeFormat::Auto.resolve(Format::Human), TimeFormat::Time);
        assert_eq!(TimeFormat::Auto.resolve(Format::Markdown), TimeFormat::Time);
        assert_eq!(TimeFormat::Auto.resolve(Format::Csv), TimeFormat::Micros);
        assert_eq!(TimeFormat::Auto.resolve(Format::Tsv), TimeFormat::Micros);
        assert_eq!(TimeFormat::Auto.resolve(Format::Json), TimeFormat::Micros);
        assert_eq!(TimeFormat::Iso.resolve(Format::Human), TimeFormat::Iso);

        assert_eq!(TimeFormat::Auto.to_string(), "auto");
        assert_eq!(TimeFormat::Time.to_string(), "time");
        assert_eq!(TimeFormat::DateTime.to_string(), "datetime");
        assert_eq!(TimeFormat::Micros.to_string(), "micros");
        assert_eq!(TimeFormat::Millis.to_string(), "millis");
        assert_eq!(TimeFormat::Seconds.to_string(), "seconds");
        assert_eq!(TimeFormat::Iso.to_string(), "iso");
        assert_eq!(TimeFormat::Elapsed.to_string(), "elapsed");
        assert_eq!(TimeFormat::Custom("%H".to_string()).to_string(), "%H");

        assert_eq!("auto".parse::<TimeFormat>().unwrap(), TimeFormat::Auto);
        assert_eq!("time".parse::<TimeFormat>().unwrap(), TimeFormat::Time);
        assert_eq!(
            "datetime".parse::<TimeFormat>().unwrap(),
            TimeFormat::DateTime
        );
        assert_eq!("micros".parse::<TimeFormat>().unwrap(), TimeFormat::Micros);
        assert_eq!("millis".parse::<TimeFormat>().unwrap(), TimeFormat::Millis);
        assert_eq!(
            "seconds".parse::<TimeFormat>().unwrap(),
            TimeFormat::Seconds
        );
        assert_eq!("iso".parse::<TimeFormat>().unwrap(), TimeFormat::Iso);
        assert_eq!(
            "elapsed".parse::<TimeFormat>().unwrap(),
            TimeFormat::Elapsed
        );
        assert_eq!(
            "%Y-%m-%d".parse::<TimeFormat>().unwrap(),
            TimeFormat::Custom("%Y-%m-%d".to_string())
        );
    }

    #[test]
    fn test_cpu_range_out_of_bounds() {
        let cpu_count = utils::get_cpu_count().unwrap();
        let out_idx = cpu_count + 10;
        let err = format!("{out_idx}").parse::<CpuRange>().unwrap_err();
        assert_eq!(
            err,
            CliError::IndexOutOfBounds {
                index: out_idx,
                total: cpu_count
            }
        );

        let err = format!("0-{out_idx}").parse::<CpuRange>().unwrap_err();
        assert_eq!(
            err,
            CliError::RangeOutOfBounds {
                start: 0,
                end: out_idx,
                total: cpu_count
            }
        );
    }

    #[test]
    fn test_cpu_range_into_inner() {
        let range = CpuRange::new(vec![0, 1]);
        assert_eq!(range.into_inner(), vec![0, 1]);
    }

    #[test]
    fn test_cli_parse_flags() {
        let cli = Cli::parse_from([
            "cpumon",
            "-n",
            "10",
            "-i",
            "500ms",
            "-d",
            "5s",
            "-f",
            "json",
            "-t",
            "iso",
            "--no-header",
            "--view",
            "cores",
            "0",
        ]);
        assert_eq!(cli.count, Some(10));
        assert_eq!(*cli.interval, std::time::Duration::from_millis(500));
        assert!(cli.duration.is_some());
        assert_eq!(cli.format, Format::Json);
        assert_eq!(cli.time_format, TimeFormat::Iso);
        assert!(cli.no_header);
        assert_eq!(cli.view, ViewMode::Cores);
        assert_eq!(cli.cpu_range.cores(), &[0]);
    }

    #[test]
    fn test_view_mode_display_and_parse() {
        assert_eq!(ViewMode::Both.to_string(), "both");
        assert_eq!(ViewMode::Cores.to_string(), "cores");
        assert_eq!(ViewMode::Avg.to_string(), "avg");
    }

    #[test]
    fn test_shell_completions_files_exist() {
        let files = [
            "completions/cpumon.bash",
            "completions/_cpumon",
            "completions/cpumon.fish",
        ];
        for file in files {
            let content = std::fs::read_to_string(file)
                .unwrap_or_else(|e| panic!("failed to read {file}: {e}"));
            assert!(!content.is_empty());
            assert!(content.contains("cpumon"));
        }
    }
}
