use std::time::{Duration, SystemTime};

use crate::error::{CpuError, CpuResult};

/// Retrieves the total number of available logical CPU cores.
///
/// # Errors
/// Returns [`CpuError::NoCoresFound`] if core affinity discovery returns no cores.
pub fn get_cpu_count() -> CpuResult<usize> {
    match core_affinity::get_core_ids() {
        Some(cores) if !cores.is_empty() => Ok(cores.len()),
        _ => Err(CpuError::NoCoresFound),
    }
}

/// Returns the current duration since the UNIX epoch.
#[must_use]
pub fn get_timestamp() -> Duration {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
}

/// Sets the CPU core affinity for the current calling thread.
///
/// # Errors
/// Returns [`CpuError::NoCoresFound`] if core affinity discovery fails,
/// or [`CpuError::FailedAffinitySet`] if `core_id` is invalid or cannot be set.
pub fn set_affinity(core_id: usize) -> CpuResult<()> {
    let Some(cores) = core_affinity::get_core_ids() else {
        return Err(CpuError::NoCoresFound);
    };

    let Some(&core) = cores.get(core_id) else {
        return Err(CpuError::FailedAffinitySet {
            core_id,
            total_cores: cores.len(),
        });
    };

    if core_affinity::set_for_current(core) {
        Ok(())
    } else {
        Err(CpuError::FailedAffinitySet {
            core_id,
            total_cores: cores.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_count_positive() {
        let count = get_cpu_count().expect("failed to get cpu count");
        assert!(count > 0);
    }

    #[test]
    fn test_timestamp_positive() {
        let ts = get_timestamp();
        assert!(!ts.is_zero());
    }

    #[test]
    fn test_affinity_valid_and_invalid() {
        assert!(set_affinity(0).is_ok());

        let invalid_core = 999_999;
        let err = set_affinity(invalid_core).unwrap_err();
        match err {
            CpuError::FailedAffinitySet { core_id, .. } => {
                assert_eq!(core_id, invalid_core);
            }
            other => panic!("expected FailedAffinitySet, got {other:?}"),
        }
    }
}
