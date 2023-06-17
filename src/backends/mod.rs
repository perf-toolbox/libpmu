pub(crate) trait BackendCounters {
    fn start(&mut self);
    fn stop(&mut self);

    fn peek(&self, id: usize) -> Option<crate::CounterValue>;
}

pub(crate) trait Backend {
    fn create_counters(
        &self,
        pid: Option<i32>,
        groups: &[crate::CountersGroup],
    ) -> Result<Box<dyn BackendCounters>, String>;
}

pub enum BackendKind {
    Perf,
    KPerf,
}

mod kperf;
mod perf;

#[cfg(target_os = "linux")]
pub(crate) use perf::PerfBackend;

#[cfg(target_os = "macos")]
pub(crate) use kperf::KPerfBackend;

pub fn get_software_events(backend: BackendKind) -> Vec<crate::SystemCounter> {
    match backend {
        BackendKind::Perf => {
            return perf::get_software_events();
        }
        BackendKind::KPerf => {
            return kperf::get_software_events();
        }
    }
}
