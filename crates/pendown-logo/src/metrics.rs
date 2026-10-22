//! Measurements that stay in the code (ADR 0020): counters and timers kept as
//! atomics, and each thread's CPU time. They're on by default in debug builds
//! and off in release; `PENDOWN_METRICS=1` or `0` says otherwise. Off, each
//! measurement is one check of a flag.
//!
//! Metrics are values, not globals: each part of Pendown keeps its own in a
//! struct, so a test can read the counts of one run while others run beside
//! it. This module has the parts they share, and the interpreter's own
//! [`InterpreterMetrics`], which live in its [`Control`](crate::Control).

use std::{
    sync::atomic::{AtomicU8, AtomicU64, Ordering},
    time::{Duration, Instant},
};

/// 0 means not decided yet, 1 off, 2 on.
static ENABLED: AtomicU8 = AtomicU8::new(0);

/// What `PENDOWN_METRICS` says, if anything.
fn from_env(value: Option<&str>) -> Option<bool> {
    match value?.trim() {
        "1" | "on" | "true" | "yes" => Some(true),
        "0" | "off" | "false" | "no" => Some(false),
        _ => None,
    }
}

/// Whether metrics are being kept.
pub fn enabled() -> bool {
    match ENABLED.load(Ordering::Relaxed) {
        1 => false,
        2 => true,
        _ => {
            let on = from_env(std::env::var("PENDOWN_METRICS").ok().as_deref()).unwrap_or(cfg!(debug_assertions));
            // Another thread may have decided first, perhaps by `set_enabled`.
            let _ = ENABLED.compare_exchange(0, if on { 2 } else { 1 }, Ordering::Relaxed, Ordering::Relaxed);
            ENABLED.load(Ordering::Relaxed) == 2
        }
    }
}

/// Turns metrics on or off from now on, as the GUI's overlay does.
pub fn set_enabled(on: bool) {
    ENABLED.store(if on { 2 } else { 1 }, Ordering::Relaxed);
}

/// A count of something.
#[derive(Debug, Default)]
pub struct Counter(AtomicU64);

impl Counter {
    pub fn add(&self, n: u64) {
        if enabled() {
            self.0.fetch_add(n, Ordering::Relaxed);
        }
    }

    pub fn get(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }

    /// For a count kept elsewhere, such as the interpreter's own count of
    /// instructions, published now and then.
    pub fn set(&self, n: u64) {
        if enabled() {
            self.0.store(n, Ordering::Relaxed);
        }
    }
}

/// Total time spent on something, and how many times.
#[derive(Debug, Default)]
pub struct Timer {
    nanos: AtomicU64,
    count: AtomicU64,
}

impl Timer {
    /// The time now, if metrics are on, to pass to `stop` later.
    pub fn start(&self) -> Option<Instant> {
        enabled().then(Instant::now)
    }

    pub fn stop(&self, started: Option<Instant>) {
        if let Some(started) = started {
            self.record(started.elapsed());
        }
    }

    pub fn record(&self, d: Duration) {
        if enabled() {
            self.nanos.fetch_add(d.as_nanos() as u64, Ordering::Relaxed);
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Times `f`.
    pub fn time<T>(&self, f: impl FnOnce() -> T) -> T {
        let started = self.start();
        let out = f();
        self.stop(started);
        out
    }

    pub fn total(&self) -> Duration {
        Duration::from_nanos(self.nanos.load(Ordering::Relaxed))
    }

    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// The average time each, or zero if there were none.
    pub fn mean(&self) -> Duration {
        self.total().checked_div(self.count().max(1) as u32).unwrap_or_default()
    }
}

/// What the interpreter measures (ADR 0020), for the front end to read.
#[derive(Debug, Default)]
pub struct InterpreterMetrics {
    /// Instructions run in this job, published every few thousand and at
    /// the end, so a program that draws nothing still shows a rate.
    pub instructions: Counter,
    /// The whole job, from parsing to the last event sent.
    pub running: Timer,
    /// Waiting in lockstep for the front end to reply to a `Sync`.
    pub lockstep: Timer,
    /// Waiting to send a batch of events into a full channel.
    pub channel: Timer,
}

/// CPU time used so far by the threads of this process, grouped.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ThreadCpu {
    /// The main thread, which is the UI thread in the GUI.
    pub main: Duration,
    /// Threads named `interpreter`.
    pub interpreter: Duration,
    /// Everything else: vello_cpu's workers, which have no names, and the
    /// windowing system's threads.
    pub other: Duration,
}

impl ThreadCpu {
    /// Read now. On platforms other than Linux, it's all zero.
    pub fn now() -> ThreadCpu {
        let mut cpu = ThreadCpu::default();
        #[cfg(target_os = "linux")]
        {
            let pid = std::process::id().to_string();
            let Ok(tasks) = std::fs::read_dir("/proc/self/task") else { return cpu };
            for task in tasks.flatten() {
                let path = task.path();
                let Some(used) = std::fs::read_to_string(path.join("schedstat")).ok().and_then(|s| schedstat(&s)) else {
                    continue;
                };
                let name = std::fs::read_to_string(path.join("comm")).unwrap_or_default();
                if task.file_name().to_str() == Some(&pid) {
                    cpu.main += used;
                } else if name.trim() == "interpreter" {
                    cpu.interpreter += used;
                } else {
                    cpu.other += used;
                }
            }
        }
        cpu
    }

    /// The CPU time used between `earlier` and now. A thread that ended
    /// between them takes its time with it, so a group can't go below zero.
    pub fn since(&self, earlier: &ThreadCpu) -> ThreadCpu {
        ThreadCpu {
            main: self.main.saturating_sub(earlier.main),
            interpreter: self.interpreter.saturating_sub(earlier.interpreter),
            other: self.other.saturating_sub(earlier.other),
        }
    }

    /// Each group as a percentage of one core over `wall`, as in
    /// `UI 35% interpreter 2% other 0%`, with `main` called `main_name`.
    pub fn percentages(&self, wall: Duration, main_name: &str) -> String {
        let percent = |d: Duration| 100.0 * d.as_secs_f64() / wall.as_secs_f64().max(1e-9);
        format!(
            "{main_name} {:.0}% interpreter {:.0}% other {:.0}%",
            percent(self.main),
            percent(self.interpreter),
            percent(self.other)
        )
    }
}

/// The calling thread's CPU time so far, for timing one thread's frames.
pub fn this_thread_cpu() -> Option<Duration> {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/thread-self/schedstat").ok().and_then(|s| schedstat(&s))
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// `schedstat`'s first field: nanoseconds spent running.
fn schedstat(text: &str) -> Option<Duration> {
    text.split_whitespace().next()?.parse().ok().map(Duration::from_nanos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_environment_variable() {
        assert_eq!(from_env(Some("1")), Some(true));
        assert_eq!(from_env(Some("0")), Some(false));
        assert_eq!(from_env(Some(" off ")), Some(false));
        assert_eq!(from_env(Some("maybe")), None);
        assert_eq!(from_env(None), None);
    }

    #[test]
    fn counters_and_timers() {
        set_enabled(true);
        let counter = Counter::default();
        counter.add(2);
        counter.add(3);
        assert_eq!(counter.get(), 5);
        let timer = Timer::default();
        timer.record(Duration::from_millis(2));
        timer.record(Duration::from_millis(4));
        assert_eq!((timer.total(), timer.count(), timer.mean()), (Duration::from_millis(6), 2, Duration::from_millis(3)));
        assert_eq!(timer.time(|| 7), 7);
        assert_eq!(timer.count(), 3);
    }

    #[test]
    fn schedstat_is_read_in_nanoseconds() {
        assert_eq!(schedstat("123456789 2000 15\n"), Some(Duration::from_nanos(123_456_789)));
        assert_eq!(schedstat(""), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_threads_cpu_time_is_found() {
        let busy = |until: Instant| {
            let mut x = 0u64;
            while Instant::now() < until {
                x = x.wrapping_add(1);
            }
            std::hint::black_box(x);
        };
        let before = this_thread_cpu().expect("Linux has /proc/thread-self");
        busy(Instant::now() + Duration::from_millis(30));
        let used = this_thread_cpu().unwrap() - before;
        assert!(used >= Duration::from_millis(20), "{used:?}");

        // A thread named `interpreter` is counted as one.
        let start = ThreadCpu::now();
        let worker = std::thread::Builder::new().name("interpreter".into()).spawn(move || {
            busy(Instant::now() + Duration::from_millis(30));
            ThreadCpu::now()
        });
        let during = worker.unwrap().join().unwrap();
        assert!(during.since(&start).interpreter >= Duration::from_millis(20), "{:?}", during.since(&start));
    }
}
