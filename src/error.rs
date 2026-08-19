use std::io;

pub use crate::cli::CliError;

pub type CpuResult<T> = Result<T, CpuError>;

#[derive(Debug, thiserror::Error)]
pub enum CpuError {
    #[error("no CPU cores found")]
    NoCoresFound,

    #[error(
        "failed to pin to core {core_id} (available: 0..{max})",
        max = total_cores.saturating_sub(1)
    )]
    FailedAffinitySet { core_id: usize, total_cores: usize },

    #[error("failed to set signal handler: {0}")]
    CtrlC(#[from] ctrlc::Error),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("{0}")]
    Cli(#[from] CliError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_error_display() {
        assert_eq!(CpuError::NoCoresFound.to_string(), "no CPU cores found");
        assert_eq!(
            CpuError::FailedAffinitySet {
                core_id: 99,
                total_cores: 8
            }
            .to_string(),
            "failed to pin to core 99 (available: 0..7)"
        );
        assert_eq!(
            CpuError::Io(io::Error::new(io::ErrorKind::NotFound, "file not found")).to_string(),
            "I/O error: file not found"
        );
    }

    #[test]
    fn test_cli_error_display() {
        assert_eq!(CliError::NoCoresFound.to_string(), "no CPU cores found");
        assert_eq!(
            CliError::InvalidCpuSpec("foo".to_string()).to_string(),
            "invalid core specification 'foo' (expected 'all', index, or '0-3')"
        );
        assert_eq!(
            CliError::EmptySpec.to_string(),
            "empty core specification in list (unexpected comma)"
        );
        assert_eq!(
            CliError::ReversedRange { start: 3, end: 1 }.to_string(),
            "invalid core range '3-1': start (3) > end (1)"
        );
        assert_eq!(
            CliError::IndexOutOfBounds {
                index: 10,
                total: 4
            }
            .to_string(),
            "core index 10 out of bounds (available: 0..3)"
        );
        assert_eq!(
            CliError::RangeOutOfBounds {
                start: 0,
                end: 10,
                total: 4
            }
            .to_string(),
            "core range 0-10 out of bounds (available: 0..3)"
        );
    }
}
