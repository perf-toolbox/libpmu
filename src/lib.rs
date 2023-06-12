mod backends;
mod events;
mod ffi;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemCounterKind {
    Software,
    Hardware,
}

#[derive(Debug, Clone)]
pub struct SystemCounter {
    pub kind: SystemCounterKind,
    pub name: &'static str,
    pub(crate) encoding: u64,
}

fn create_backend(kind: backends::BackendKind) -> Result<Box<dyn backends::Backend>, String> {
    match kind {
        backends::BackendKind::Perf => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "linux")] {
                   Ok(Box::new(backends::PerfBackend::new()))
                } else {
                    Err("Backend not supported for current OS".to_string())
                }
            }
        }
        backends::BackendKind::KPerf => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                   Ok(Box::new(backends::KPerfBackend::new()))
                } else {
                    Err("Backend not supported for current OS".to_string())
                }
            }
        }
    }
}

fn create_default_backend() -> Result<Box<dyn backends::Backend>, String> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            create_backend(backends::BackendKind::Perf)
        } else if #[cfg(target_os = "macos")] {
            create_backend(backends::BackendKind::KPerf)
        } else {
            Err("Unsupported OS".to_string())
        }
    }
}

pub fn list_events_for_backend(kind: backends::BackendKind) -> Vec<SystemCounter> {
    let mut events = backends::get_software_events(kind);

    let hw_events = events::get_hardware_events();
    events.extend(hw_events);

    return events;
}

pub fn list_events() -> Vec<SystemCounter> {
    return list_events_for_backend(backends::BackendKind::Perf);
}

pub fn find_event_by_name(name: &str) -> Option<SystemCounter> {
    for e in list_events() {
        if e.to_string() == name {
            return Some(e);
        }
    }

    return None;
}

#[derive(Debug, Clone)]
pub enum CounterKind {
    Cycles,
    Instructions,
    CacheHits,
    CacheMisses,
    Branches,
    BranchMisses,
    System(SystemCounter),
}

pub struct Builder {
    backend: Box<dyn backends::Backend>,
    pid: Option<i32>,
    counters: Vec<CounterKind>,
    period: u32,
    callback: Option<Box<dyn Fn() -> ()>>,
}

pub struct Counters {
    backend_counters: Box<dyn backends::BackendCounters>,
}

pub struct CountersIterator<'a> {
    cur: usize,
    backend_counters: &'a Box<dyn backends::BackendCounters>,
}

pub struct CounterValue {
    pub kind: CounterKind,
    pub value: usize,
}

impl Builder {
    fn default(backend: Box<dyn backends::Backend>) -> Builder {
        return Builder {
            backend,
            pid: None,
            counters: vec![],
            period: 0,
            callback: None,
        };
    }

    pub fn new() -> Builder {
        return Builder::default(create_default_backend().unwrap());
    }

    pub fn new_from_backend(backend: backends::BackendKind) -> Result<Builder, String> {
        return Ok(Builder::default(create_backend(backend)?));
    }

    pub fn attach_pid(&mut self, pid: i32) {
        self.pid = Some(pid);
    }

    pub fn attach(&mut self, child: std::process::Child) {
        self.pid = Some(child.id() as i32);
    }

    pub fn enable_sampling(&mut self, period: u32, callback: Box<dyn Fn() -> ()>) {
        self.period = period;
        self.callback = Some(callback);
    }

    pub fn add_counter(&mut self, counter: CounterKind) {
        self.counters.push(counter);
    }

    pub fn build(&self) -> Result<Counters, String> {
        let backend_counters =
            self.backend
                .create_counters(self.pid, self.period, &self.counters)?;
        return Ok(Counters { backend_counters });
    }
}

impl Counters {
    pub fn start(&mut self) {
        self.backend_counters.start();
    }
    pub fn stop(&mut self) {
        self.backend_counters.stop();
    }

    pub fn iter<'a>(&'a self) -> CountersIterator<'a> {
        return CountersIterator {
            cur: 0,
            backend_counters: &self.backend_counters,
        };
    }
}

impl Iterator for CountersIterator<'_> {
    type Item = CounterValue;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.cur;
        self.cur += 1;
        return self.backend_counters.peek(current);
    }
}

impl ToString for SystemCounter {
    fn to_string(&self) -> String {
        let prefix = match self.kind {
            SystemCounterKind::Software => "SW",
            SystemCounterKind::Hardware => "HW",
        };

        return format!("{}:{}", prefix, self.name).into();
    }
}

impl ToString for CounterKind {
    fn to_string(&self) -> String {
        match self {
            CounterKind::Cycles => "cycles".into(),
            CounterKind::Instructions => "instructions".into(),
            CounterKind::Branches => "branches".into(),
            CounterKind::BranchMisses => "branch_misses".into(),
            CounterKind::CacheMisses => "cache_misses".into(),
            CounterKind::System(counter) => counter.to_string(),
            _ => unimplemented!(),
        }
    }
}

impl PartialEq for SystemCounter {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.name == other.name
    }
}

impl Eq for SystemCounter {}
