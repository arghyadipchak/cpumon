use std::{
    io::{self, Write},
    time::{Duration, SystemTime},
};

use chrono::{DateTime, Local, Utc};

use crate::{
    cli::{Format, TimeFormat, ViewMode},
    sample::CpuSample,
};

/// Prints the column header for the given output format.
///
/// # Errors
/// Returns an [`io::Error`] if writing to the output stream fails.
pub fn print_header<W: Write>(
    mut writer: W,
    format: Format,
    time_format: &TimeFormat,
    cores: &[usize],
    view: ViewMode,
) -> io::Result<()> {
    let show_avg = view != ViewMode::Cores;
    let show_cores = view != ViewMode::Avg;

    match format {
        Format::Json => Ok(()),
        Format::Human => {
            let resolved = time_format.resolve(format);
            let time_width = time_column_width(&resolved, 12);
            if show_avg {
                write!(
                    writer,
                    "{:<width$} {:>7}",
                    "TIME",
                    "AVG",
                    width = time_width
                )?;
            } else {
                write!(writer, "{:<width$}", "TIME", width = time_width)?;
            }
            if show_cores {
                for &core in cores {
                    write!(writer, " {:>7}", format!("CPU {core}"))?;
                }
            }
            writeln!(writer)
        }
        Format::Csv => {
            if show_avg {
                write!(writer, "timestamp,avg")?;
            } else {
                write!(writer, "timestamp")?;
            }
            if show_cores {
                for &core in cores {
                    write!(writer, ",core_{core}")?;
                }
            }
            writeln!(writer)
        }
        Format::Tsv => {
            if show_avg {
                write!(writer, "timestamp\tavg")?;
            } else {
                write!(writer, "timestamp")?;
            }
            if show_cores {
                for &core in cores {
                    write!(writer, "\tcore_{core}")?;
                }
            }
            writeln!(writer)
        }
        Format::Markdown => {
            if show_avg {
                write!(writer, "| TIME | AVG |")?;
            } else {
                write!(writer, "| TIME |")?;
            }
            if show_cores {
                for &core in cores {
                    write!(writer, " CPU {core} |")?;
                }
            }
            writeln!(writer)?;
            if show_avg {
                write!(writer, "| :--- | :---: |")?;
            } else {
                write!(writer, "| :--- |")?;
            }
            if show_cores {
                for _ in cores {
                    write!(writer, " :---: |")?;
                }
            }
            writeln!(writer)
        }
    }
}

/// Prints a CPU sample to the provided writer in the specified format.
///
/// # Errors
/// Returns an [`io::Error`] if writing to the output stream fails.
pub fn print_sample<W: Write>(
    mut writer: W,
    format: Format,
    time_format: &TimeFormat,
    sample: &CpuSample,
    view: ViewMode,
) -> io::Result<()> {
    let resolved_time_fmt = time_format.resolve(format);
    let show_avg = view != ViewMode::Cores;
    let show_cores = view != ViewMode::Avg;

    match format {
        Format::Human => {
            let time_str =
                format_timestamp(sample.timestamp, sample.start_timestamp, &resolved_time_fmt);
            let time_width = time_column_width(&resolved_time_fmt, time_str.len());
            if show_avg {
                write!(writer, "{time_str:<time_width$} {:>6.1}%", sample.avg)?;
            } else {
                write!(writer, "{time_str:<time_width$}")?;
            }
            if show_cores {
                for (_, usage) in &sample.cores {
                    write!(writer, " {usage:>6.1}%")?;
                }
            }
            writeln!(writer)
        }
        Format::Csv => {
            write_timestamp(
                &mut writer,
                sample.timestamp,
                sample.start_timestamp,
                &resolved_time_fmt,
            )?;
            if show_avg {
                write!(writer, ",{:.2}", sample.avg)?;
            }
            if show_cores {
                for (_, usage) in &sample.cores {
                    write!(writer, ",{usage:.2}")?;
                }
            }
            writeln!(writer)
        }
        Format::Tsv => {
            write_timestamp(
                &mut writer,
                sample.timestamp,
                sample.start_timestamp,
                &resolved_time_fmt,
            )?;
            if show_avg {
                write!(writer, "\t{:.2}", sample.avg)?;
            }
            if show_cores {
                for (_, usage) in &sample.cores {
                    write!(writer, "\t{usage:.2}")?;
                }
            }
            writeln!(writer)
        }
        Format::Markdown => {
            let time_str =
                format_timestamp(sample.timestamp, sample.start_timestamp, &resolved_time_fmt);
            if show_avg {
                write!(writer, "| {time_str} | {:>5.1}% |", sample.avg)?;
            } else {
                write!(writer, "| {time_str} |")?;
            }
            if show_cores {
                for (_, usage) in &sample.cores {
                    write!(writer, " {usage:>5.1}% |")?;
                }
            }
            writeln!(writer)
        }
        Format::Json => print_sample_json(writer, &resolved_time_fmt, sample, view),
    }
}

/// Serializes and writes a single sample as line-delimited JSON.
fn print_sample_json<W: Write>(
    mut writer: W,
    resolved_time_fmt: &TimeFormat,
    sample: &CpuSample,
    view: ViewMode,
) -> io::Result<()> {
    let is_numeric = matches!(
        resolved_time_fmt,
        TimeFormat::Micros | TimeFormat::Millis | TimeFormat::Seconds
    );
    if is_numeric {
        write!(writer, "{{\"timestamp\":")?;
        write_timestamp(
            &mut writer,
            sample.timestamp,
            sample.start_timestamp,
            resolved_time_fmt,
        )?;
    } else {
        write!(writer, "{{\"timestamp\":\"")?;
        write_timestamp(
            &mut writer,
            sample.timestamp,
            sample.start_timestamp,
            resolved_time_fmt,
        )?;
        write!(writer, "\"")?;
    }

    match view {
        ViewMode::Both => {
            write!(writer, ",\"avg\":{:.2},\"cores\":{{", sample.avg)?;
            for (i, &(core, usage)) in sample.cores.iter().enumerate() {
                if i > 0 {
                    write!(writer, ",")?;
                }
                write!(writer, "\"{core}\":{usage:.2}")?;
            }
            writeln!(writer, "}}}}")
        }
        ViewMode::Cores => {
            write!(writer, ",\"cores\":{{")?;
            for (i, &(core, usage)) in sample.cores.iter().enumerate() {
                if i > 0 {
                    write!(writer, ",")?;
                }
                write!(writer, "\"{core}\":{usage:.2}")?;
            }
            writeln!(writer, "}}}}")
        }
        ViewMode::Avg => {
            writeln!(writer, ",\"avg\":{:.2}}}", sample.avg)
        }
    }
}

/// Writes formatted timestamp directly into the destination writer without intermediate string allocation.
///
/// # Errors
/// Returns an [`io::Error`] if writing fails.
pub fn write_timestamp<W: Write>(
    mut writer: W,
    timestamp: Duration,
    start_timestamp: Duration,
    format: &TimeFormat,
) -> io::Result<()> {
    match format {
        TimeFormat::Micros | TimeFormat::Auto => write!(writer, "{}", timestamp.as_micros()),
        TimeFormat::Millis => write!(writer, "{}", timestamp.as_millis()),
        TimeFormat::Seconds => write!(writer, "{}", timestamp.as_secs()),
        TimeFormat::Time => {
            let local_dt: DateTime<Local> = DateTime::from(SystemTime::UNIX_EPOCH + timestamp);
            write!(writer, "{}", local_dt.format("%H:%M:%S%.3f"))
        }
        TimeFormat::DateTime => {
            let local_dt: DateTime<Local> = DateTime::from(SystemTime::UNIX_EPOCH + timestamp);
            write!(writer, "{}", local_dt.format("%Y-%m-%d %H:%M:%S"))
        }
        TimeFormat::Iso => {
            let utc_dt: DateTime<Utc> = DateTime::from(SystemTime::UNIX_EPOCH + timestamp);
            write!(writer, "{}", utc_dt.format("%Y-%m-%dT%H:%M:%S%.3fZ"))
        }
        TimeFormat::Elapsed => write_elapsed(writer, timestamp.saturating_sub(start_timestamp)),
        TimeFormat::Custom(pattern) => {
            let local_dt: DateTime<Local> = DateTime::from(SystemTime::UNIX_EPOCH + timestamp);
            write!(writer, "{}", local_dt.format(pattern))
        }
    }
}

/// Formats a timestamp Duration into a human-readable or machine string.
#[must_use]
pub fn format_timestamp(
    timestamp: Duration,
    start_timestamp: Duration,
    format: &TimeFormat,
) -> String {
    let mut buf = Vec::with_capacity(32);
    let _ = write_timestamp(&mut buf, timestamp, start_timestamp, format);
    String::from_utf8(buf).unwrap_or_default()
}

fn time_column_width(format: &TimeFormat, sample_len: usize) -> usize {
    match format {
        TimeFormat::Iso => 24,
        TimeFormat::DateTime => 19,
        TimeFormat::Micros => 16,
        TimeFormat::Millis => 13,
        TimeFormat::Seconds => 10,
        TimeFormat::Time | TimeFormat::Elapsed | TimeFormat::Auto => 12,
        TimeFormat::Custom(_) => sample_len.max(8),
    }
}

fn write_elapsed<W: Write>(mut writer: W, elapsed: Duration) -> io::Result<()> {
    let total_secs = elapsed.as_secs();
    let millis = elapsed.subsec_millis();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    write!(writer, "{hours:02}:{minutes:02}:{seconds:02}.{millis:03}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_fixture() -> CpuSample {
        CpuSample::new(
            Duration::from_secs(1_700_000_000),
            Duration::from_secs(1_700_000_000),
            12.5,
            vec![(0, 15.0), (1, 10.0)],
        )
    }

    #[test]
    fn test_csv_header_formatting() {
        let mut buffer = Vec::new();
        print_header(
            &mut buffer,
            Format::Csv,
            &TimeFormat::Auto,
            &[0, 1, 3],
            ViewMode::Both,
        )
        .unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "timestamp,avg,core_0,core_1,core_3\n");

        let mut buffer_cores = Vec::new();
        print_header(
            &mut buffer_cores,
            Format::Csv,
            &TimeFormat::Auto,
            &[0, 1, 3],
            ViewMode::Cores,
        )
        .unwrap();
        let output_cores = String::from_utf8(buffer_cores).unwrap();
        assert_eq!(output_cores, "timestamp,core_0,core_1,core_3\n");

        let mut buffer_avg = Vec::new();
        print_header(
            &mut buffer_avg,
            Format::Csv,
            &TimeFormat::Auto,
            &[0, 1, 3],
            ViewMode::Avg,
        )
        .unwrap();
        let output_avg = String::from_utf8(buffer_avg).unwrap();
        assert_eq!(output_avg, "timestamp,avg\n");
    }

    #[test]
    fn test_human_header_formatting() {
        let mut buffer = Vec::new();
        print_header(
            &mut buffer,
            Format::Human,
            &TimeFormat::Auto,
            &[0, 1],
            ViewMode::Both,
        )
        .unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "TIME             AVG   CPU 0   CPU 1\n");

        let mut buffer_cores = Vec::new();
        print_header(
            &mut buffer_cores,
            Format::Human,
            &TimeFormat::Auto,
            &[0, 1],
            ViewMode::Cores,
        )
        .unwrap();
        let output_cores = String::from_utf8(buffer_cores).unwrap();
        assert_eq!(output_cores, "TIME           CPU 0   CPU 1\n");

        let mut buffer_avg = Vec::new();
        print_header(
            &mut buffer_avg,
            Format::Human,
            &TimeFormat::Auto,
            &[0, 1],
            ViewMode::Avg,
        )
        .unwrap();
        let output_avg = String::from_utf8(buffer_avg).unwrap();
        assert_eq!(output_avg, "TIME             AVG\n");
    }

    #[test]
    fn test_tsv_header_formatting() {
        let mut buffer = Vec::new();
        print_header(
            &mut buffer,
            Format::Tsv,
            &TimeFormat::Auto,
            &[0, 1],
            ViewMode::Both,
        )
        .unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "timestamp\tavg\tcore_0\tcore_1\n");

        let mut buffer_cores = Vec::new();
        print_header(
            &mut buffer_cores,
            Format::Tsv,
            &TimeFormat::Auto,
            &[0, 1],
            ViewMode::Cores,
        )
        .unwrap();
        let output_cores = String::from_utf8(buffer_cores).unwrap();
        assert_eq!(output_cores, "timestamp\tcore_0\tcore_1\n");

        let mut buffer_avg = Vec::new();
        print_header(
            &mut buffer_avg,
            Format::Tsv,
            &TimeFormat::Auto,
            &[0, 1],
            ViewMode::Avg,
        )
        .unwrap();
        let output_avg = String::from_utf8(buffer_avg).unwrap();
        assert_eq!(output_avg, "timestamp\tavg\n");
    }

    #[test]
    fn test_markdown_header_formatting() {
        let mut buffer = Vec::new();
        print_header(
            &mut buffer,
            Format::Markdown,
            &TimeFormat::Auto,
            &[0, 1],
            ViewMode::Both,
        )
        .unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.starts_with("| TIME | AVG | CPU 0 | CPU 1 |\n"));
        assert!(output.contains("| :--- | :---: | :---: | :---: |\n"));

        let mut buffer_cores = Vec::new();
        print_header(
            &mut buffer_cores,
            Format::Markdown,
            &TimeFormat::Auto,
            &[0, 1],
            ViewMode::Cores,
        )
        .unwrap();
        let output_cores = String::from_utf8(buffer_cores).unwrap();
        assert!(output_cores.starts_with("| TIME | CPU 0 | CPU 1 |\n"));
        assert!(output_cores.contains("| :--- | :---: | :---: |\n"));

        let mut buffer_avg = Vec::new();
        print_header(
            &mut buffer_avg,
            Format::Markdown,
            &TimeFormat::Auto,
            &[0, 1],
            ViewMode::Avg,
        )
        .unwrap();
        let output_avg = String::from_utf8(buffer_avg).unwrap();
        assert!(output_avg.starts_with("| TIME | AVG |\n"));
        assert!(output_avg.contains("| :--- | :---: |\n"));
    }

    #[test]
    fn test_csv_print_formatting_micros() {
        let mut sample = sample_fixture();
        sample.timestamp = Duration::from_micros(123_456_789);
        let mut buffer = Vec::new();
        print_sample(
            &mut buffer,
            Format::Csv,
            &TimeFormat::Micros,
            &sample,
            ViewMode::Both,
        )
        .unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "123456789,12.50,15.00,10.00\n");

        let mut buffer_cores = Vec::new();
        print_sample(
            &mut buffer_cores,
            Format::Csv,
            &TimeFormat::Micros,
            &sample,
            ViewMode::Cores,
        )
        .unwrap();
        let output_cores = String::from_utf8(buffer_cores).unwrap();
        assert_eq!(output_cores, "123456789,15.00,10.00\n");

        let mut buffer_avg = Vec::new();
        print_sample(
            &mut buffer_avg,
            Format::Csv,
            &TimeFormat::Micros,
            &sample,
            ViewMode::Avg,
        )
        .unwrap();
        let output_avg = String::from_utf8(buffer_avg).unwrap();
        assert_eq!(output_avg, "123456789,12.50\n");
    }

    #[test]
    fn test_csv_print_formatting_iso() {
        let sample = sample_fixture();
        let mut buffer = Vec::new();
        print_sample(
            &mut buffer,
            Format::Csv,
            &TimeFormat::Iso,
            &sample,
            ViewMode::Both,
        )
        .unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "2023-11-14T22:13:20.000Z,12.50,15.00,10.00\n");
    }

    #[test]
    fn test_tsv_print_formatting() {
        let mut sample = sample_fixture();
        sample.timestamp = Duration::from_micros(123_456_789);
        let mut buffer = Vec::new();
        print_sample(
            &mut buffer,
            Format::Tsv,
            &TimeFormat::Micros,
            &sample,
            ViewMode::Both,
        )
        .unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "123456789\t12.50\t15.00\t10.00\n");

        let mut buffer_cores = Vec::new();
        print_sample(
            &mut buffer_cores,
            Format::Tsv,
            &TimeFormat::Micros,
            &sample,
            ViewMode::Cores,
        )
        .unwrap();
        let output_cores = String::from_utf8(buffer_cores).unwrap();
        assert_eq!(output_cores, "123456789\t15.00\t10.00\n");

        let mut buffer_avg = Vec::new();
        print_sample(
            &mut buffer_avg,
            Format::Tsv,
            &TimeFormat::Micros,
            &sample,
            ViewMode::Avg,
        )
        .unwrap();
        let output_avg = String::from_utf8(buffer_avg).unwrap();
        assert_eq!(output_avg, "123456789\t12.50\n");
    }

    #[test]
    fn test_markdown_print_formatting() {
        let sample = sample_fixture();
        let mut buffer = Vec::new();
        print_sample(
            &mut buffer,
            Format::Markdown,
            &TimeFormat::Time,
            &sample,
            ViewMode::Both,
        )
        .unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.starts_with("| "));
        assert!(output.contains("12.5%"));
        assert!(output.ends_with(" |\n"));

        let mut buffer_cores = Vec::new();
        print_sample(
            &mut buffer_cores,
            Format::Markdown,
            &TimeFormat::Time,
            &sample,
            ViewMode::Cores,
        )
        .unwrap();
        let output_cores = String::from_utf8(buffer_cores).unwrap();
        assert!(!output_cores.contains("12.5%"));
    }

    #[test]
    fn test_json_print_formatting() {
        let mut sample = sample_fixture();
        sample.timestamp = Duration::from_micros(123_456_789);
        let mut buffer = Vec::new();
        print_sample(
            &mut buffer,
            Format::Json,
            &TimeFormat::Auto,
            &sample,
            ViewMode::Both,
        )
        .unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(
            output,
            "{\"timestamp\":123456789,\"avg\":12.50,\"cores\":{\"0\":15.00,\"1\":10.00}}\n"
        );

        let mut buffer_cores = Vec::new();
        print_sample(
            &mut buffer_cores,
            Format::Json,
            &TimeFormat::Auto,
            &sample,
            ViewMode::Cores,
        )
        .unwrap();
        let output_cores = String::from_utf8(buffer_cores).unwrap();
        assert_eq!(
            output_cores,
            "{\"timestamp\":123456789,\"cores\":{\"0\":15.00,\"1\":10.00}}\n"
        );

        let mut buffer_avg = Vec::new();
        print_sample(
            &mut buffer_avg,
            Format::Json,
            &TimeFormat::Auto,
            &sample,
            ViewMode::Avg,
        )
        .unwrap();
        let output_avg = String::from_utf8(buffer_avg).unwrap();
        assert_eq!(output_avg, "{\"timestamp\":123456789,\"avg\":12.50}\n");
    }

    #[test]
    fn test_json_print_formatting_iso() {
        let sample = sample_fixture();
        let mut buffer = Vec::new();
        print_sample(
            &mut buffer,
            Format::Json,
            &TimeFormat::Iso,
            &sample,
            ViewMode::Both,
        )
        .unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(
            output,
            "{\"timestamp\":\"2023-11-14T22:13:20.000Z\",\"avg\":12.50,\"cores\":{\"0\":15.00,\"1\":10.00}}\n"
        );

        let mut buffer_cores = Vec::new();
        print_sample(
            &mut buffer_cores,
            Format::Json,
            &TimeFormat::Iso,
            &sample,
            ViewMode::Cores,
        )
        .unwrap();
        let output_cores = String::from_utf8(buffer_cores).unwrap();
        assert_eq!(
            output_cores,
            "{\"timestamp\":\"2023-11-14T22:13:20.000Z\",\"cores\":{\"0\":15.00,\"1\":10.00}}\n"
        );
    }

    #[test]
    fn test_human_print_formatting() {
        let sample = sample_fixture();
        let mut buffer = Vec::new();
        print_sample(
            &mut buffer,
            Format::Human,
            &TimeFormat::Time,
            &sample,
            ViewMode::Both,
        )
        .unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains(':'));
        assert!(output.ends_with("%\n"));

        let mut buffer_cores = Vec::new();
        print_sample(
            &mut buffer_cores,
            Format::Human,
            &TimeFormat::Time,
            &sample,
            ViewMode::Cores,
        )
        .unwrap();
        let output_cores = String::from_utf8(buffer_cores).unwrap();
        assert!(!output_cores.contains("12.5%"));
    }

    #[test]
    fn test_human_print_formatting_datetime() {
        let sample = sample_fixture();
        let mut buffer = Vec::new();
        print_sample(
            &mut buffer,
            Format::Human,
            &TimeFormat::DateTime,
            &sample,
            ViewMode::Both,
        )
        .unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains('-'));
        assert!(output.contains(':'));
    }

    #[test]
    fn test_csv_print_formatting_custom_strftime() {
        let sample = sample_fixture();
        let mut buffer = Vec::new();
        print_sample(
            &mut buffer,
            Format::Csv,
            &TimeFormat::Custom("%s".to_string()),
            &sample,
            ViewMode::Both,
        )
        .unwrap();
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "1700000000,12.50,15.00,10.00\n");
    }

    #[test]
    fn test_json_header_is_empty() {
        let mut buffer = Vec::new();
        print_header(
            &mut buffer,
            Format::Json,
            &TimeFormat::Auto,
            &[0, 1],
            ViewMode::Both,
        )
        .unwrap();
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_timestamp_variants_formatting() {
        let ts = Duration::from_secs(1_700_000_000) + Duration::from_millis(500);
        let start = Duration::from_secs(1_700_000_000);

        assert_eq!(
            format_timestamp(ts, start, &TimeFormat::Millis),
            "1700000000500"
        );
        assert_eq!(
            format_timestamp(ts, start, &TimeFormat::Seconds),
            "1700000000"
        );
        assert_eq!(
            format_timestamp(ts, start, &TimeFormat::Elapsed),
            "00:00:00.500"
        );

        let long_elapsed = start + Duration::from_secs(3600 + 120 + 5) + Duration::from_millis(123);
        assert_eq!(
            format_timestamp(long_elapsed, start, &TimeFormat::Elapsed),
            "01:02:05.123"
        );
    }

    #[test]
    fn test_time_column_widths() {
        assert_eq!(time_column_width(&TimeFormat::Iso, 24), 24);
        assert_eq!(time_column_width(&TimeFormat::DateTime, 19), 19);
        assert_eq!(time_column_width(&TimeFormat::Micros, 16), 16);
        assert_eq!(time_column_width(&TimeFormat::Millis, 13), 13);
        assert_eq!(time_column_width(&TimeFormat::Seconds, 10), 10);
        assert_eq!(time_column_width(&TimeFormat::Time, 12), 12);
        assert_eq!(time_column_width(&TimeFormat::Elapsed, 12), 12);
        assert_eq!(
            time_column_width(&TimeFormat::Custom("%H".to_string()), 5),
            8
        );
        assert_eq!(
            time_column_width(&TimeFormat::Custom("%Y-%m-%d %H:%M:%S".to_string()), 19),
            19
        );
    }
}
