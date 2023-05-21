pub(crate) trait BackendCounters {
    fn start(&mut self);
    fn stop(&mut self);

    fn peek(&self, id: usize) -> Option<crate::CounterValue>;
}

pub(crate) trait Backend {
    fn create_counters(
        &self,
        pid: Option<i32>,
        period: u32,
        counters: &[crate::CounterKind],
    ) -> Result<Box<dyn BackendCounters>, String>;
}

pub enum BackendKind {
    Perf,
}

mod perf;
pub(crate) use perf::PerfBackend;
