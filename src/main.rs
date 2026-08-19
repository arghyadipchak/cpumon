mod cli;
mod error;
mod format;
mod monitor;
mod sample;
mod utils;

use std::{
    io::{self, Write},
    process::ExitCode,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Instant,
};

use clap::Parser as _;

use crate::{
    cli::Cli,
    error::{CpuError, CpuResult},
    format::{print_header, print_sample},
    monitor::CpuMonitor,
    utils::set_affinity,
};

fn run() -> CpuResult<()> {
    let cli = Cli::parse();

    if let Some(core_id) = cli.cpuset {
        set_affinity(core_id)?;
    }

    let cont = Arc::new(AtomicBool::new(true));

    {
        let cont = cont.clone();
        ctrlc::set_handler(move || {
            cont.store(false, Ordering::Release);
        })?;
    }

    let mut monitor = CpuMonitor::from(cli.cpu_range);
    let stdout = io::stdout();

    if !cli.no_header {
        let mut lock = stdout.lock();
        print_header(
            &mut lock,
            cli.format,
            &cli.time_format,
            monitor.cores(),
            cli.view,
        )?;
        lock.flush()?;
    }

    // Set up monotonic clock references for drift-compensated interval scheduling
    let interval = *cli.interval;
    let start_instant = Instant::now();
    let max_duration = cli.duration.as_ref().map(|d| **d);
    let mut next_tick = start_instant + interval;
    let mut samples_left = cli.count;

    while cont.load(Ordering::Relaxed) {
        if samples_left == Some(0) {
            break;
        }
        if let Some(max_dur) = max_duration
            && start_instant.elapsed() >= max_dur
        {
            break;
        }

        // Sleep until the exact scheduled monotonic tick
        let now = Instant::now();
        if next_tick > now {
            let sleep_dur = if let Some(max_dur) = max_duration {
                let remaining = max_dur.saturating_sub(start_instant.elapsed());
                if remaining.is_zero() {
                    break;
                }
                (next_tick - now).min(remaining)
            } else {
                next_tick - now
            };
            thread::sleep(sleep_dur);
        }

        if !cont.load(Ordering::Relaxed) {
            break;
        }

        // Collect point-in-time metrics and flush directly to stdout
        let sample = monitor.sample();
        {
            let mut lock = stdout.lock();
            print_sample(&mut lock, cli.format, &cli.time_format, sample, cli.view)?;
            lock.flush()?;
        }

        if let Some(ref mut count) = samples_left {
            *count -= 1;
            if *count == 0 {
                break;
            }
        }

        if let Some(max_dur) = max_duration
            && start_instant.elapsed() >= max_dur
        {
            break;
        }

        // Advance next tick and compensate for any processing delay
        next_tick += interval;
        if next_tick <= Instant::now() {
            next_tick = Instant::now() + interval;
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(CpuError::Io(err)) if err.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
