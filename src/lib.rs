mod backends;

fn create_backend(kind: backends::BackendKind) -> Box<dyn backends::Backend> {
    match kind {
        backends::BackendKind::Perf => return Box::new(backends::PerfBackend::new()),
    }
}

fn create_default_backend() -> Box<dyn backends::Backend> {
    return create_backend(backends::BackendKind::Perf);
}

#[derive(Debug, Clone)]
pub enum CounterType {
    Cycles,
    Instructions,
    CacheMisses,
    Branches,
    Hardware(String),
}

pub struct Builder {
    backend: Box<dyn backends::Backend>,
    pid: Option<i32>,
    counters: Vec<CounterType>,
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
    pub kind: CounterType,
    pub value: usize,
}

impl Builder {
    fn default(backend: Box<dyn backends::Backend>) -> Builder {
        return Builder {
            backend: backend,
            pid: None,
            counters: vec![],
            period: 0,
            callback: None,
        };
    }

    pub fn new() -> Builder {
        return Builder::default(create_default_backend());
    }

    pub fn new_from_backend(backend: backends::BackendKind) -> Builder {
        return Builder::default(create_backend(backend));
    }

    pub fn attach(&mut self, pid: i32) {
        self.pid = Some(pid);
    }

    pub fn enable_sampling(&mut self, period: u32, callback: Box<dyn Fn() -> ()>) {
        self.period = period;
        self.callback = Some(callback);
    }

    pub fn add_counter(&mut self, counter: CounterType) {
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

impl ToString for CounterType {
    fn to_string(&self) -> String {
        match self {
            CounterType::Cycles => "cycles".into(),
            CounterType::Instructions => "instructions".into(),
            _ => unimplemented!(),
        }
    }
}
