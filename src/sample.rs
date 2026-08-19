use std::time::Duration;

/// A single point-in-time snapshot of CPU utilization metrics across monitored cores.
#[derive(Clone, Debug, PartialEq)]
pub struct CpuSample {
    /// Timestamp of this sample relative to UNIX epoch.
    pub timestamp: Duration,
    /// Timestamp when monitoring started.
    pub start_timestamp: Duration,
    /// Aggregate average CPU utilization across monitored cores (0.0 to 100.0).
    pub avg: f32,
    /// Monitored cores with their utilization percentages: (`core_id`, `usage_percent`).
    pub cores: Vec<(usize, f32)>,
}

impl CpuSample {
    /// Creates a new CPU sample snapshot.
    #[must_use]
    pub const fn new(
        timestamp: Duration,
        start_timestamp: Duration,
        avg: f32,
        cores: Vec<(usize, f32)>,
    ) -> Self {
        Self {
            timestamp,
            start_timestamp,
            avg,
            cores,
        }
    }
}
