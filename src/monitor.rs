use std::time::Duration;

use sysinfo::{CpuRefreshKind, RefreshKind, System};

use crate::{cli::CpuRange, sample::CpuSample, utils};

/// Monitors system CPU utilization across a specified set of logical cores.
pub struct CpuMonitor {
    start_timestamp: Duration,
    timestamp: Duration,
    system: System,
    cores: Vec<usize>,
    current_sample: CpuSample,
}

impl CpuMonitor {
    /// Creates a new `CpuMonitor` targeting the specified CPU cores.
    #[must_use]
    pub fn new(cores: Vec<usize>) -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
        );
        system.refresh_cpu_usage();
        let now = utils::get_timestamp();
        let initial_cores = cores.iter().map(|&c| (c, 0.0)).collect();

        Self {
            start_timestamp: now,
            timestamp: now,
            system,
            cores,
            current_sample: CpuSample::new(now, now, 0.0, initial_cores),
        }
    }

    /// Returns the slice of CPU core IDs being monitored.
    #[must_use]
    pub fn cores(&self) -> &[usize] {
        &self.cores
    }

    /// Refreshes CPU metrics from the kernel and updates in-place the point-in-time [`CpuSample`].
    #[allow(clippy::cast_precision_loss)]
    pub fn sample(&mut self) -> &CpuSample {
        self.system.refresh_cpu_usage();
        self.timestamp = utils::get_timestamp();

        let (total_usage, count) = self.cores.iter().fold((0.0f32, 0usize), |(acc, n), &core| {
            self.system
                .cpus()
                .get(core)
                .map_or((acc, n), |cpu| (acc + cpu.cpu_usage(), n + 1))
        });

        self.current_sample.timestamp = self.timestamp;
        self.current_sample.start_timestamp = self.start_timestamp;
        self.current_sample.avg = if count > 0 {
            total_usage / count as f32
        } else {
            0.0
        };

        for (slot, &core) in self.current_sample.cores.iter_mut().zip(&self.cores) {
            slot.0 = core;
            slot.1 = self
                .system
                .cpus()
                .get(core)
                .map_or(0.0, sysinfo::Cpu::cpu_usage);
        }

        &self.current_sample
    }
}

impl From<CpuRange> for CpuMonitor {
    fn from(range: CpuRange) -> Self {
        Self::new(range.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_sampling() {
        let mut monitor = CpuMonitor::new(vec![0]);
        assert_eq!(monitor.cores(), &[0]);
        let sample = monitor.sample();
        assert_eq!(sample.cores.len(), 1);
        assert_eq!(sample.cores[0].0, 0);
        assert!(sample.avg >= 0.0);
    }

    #[test]
    fn test_monitor_from_cpu_range() {
        let range = CpuRange::new(vec![0]);
        let monitor = CpuMonitor::from(range);
        assert_eq!(monitor.cores(), &[0]);
    }
}
