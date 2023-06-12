#[cfg(target_os = "linux")]
use crate::backends::{Backend, BackendCounters};
#[cfg(target_os = "linux")]
use crate::CounterKind;
use crate::{SystemCounter, SystemCounterKind};
#[cfg(target_os = "linux")]
use libc::{ptrace, read};
use perf_event_open_sys as sys;

#[cfg(target_os = "linux")]
pub(crate) struct PerfBackend {}

#[cfg(target_os = "linux")]
#[repr(C)]
struct RFValues {
    value: u64,
    id: u64,
}

#[cfg(target_os = "linux")]
struct NativeCounterHandle {
    pub kind: CounterKind,
    pub fd: i32,
    pub id: u64,
}

#[cfg(target_os = "linux")]
struct PerfCounterGroup {
    native_handles: Vec<NativeCounterHandle>,
    buffer: Vec<u8>,
}

#[cfg(target_os = "linux")]
struct PerfCounters {
    groups: Vec<PerfCounterGroup>,
    pid: i32,
}

#[cfg(target_os = "linux")]
impl PerfBackend {
    pub fn new() -> PerfBackend {
        return PerfBackend {};
    }
}

#[cfg(target_os = "linux")]
impl Backend for PerfBackend {
    fn create_counters(
        &self,
        pid: Option<i32>,
        _period: u32,
        counters: &[CounterKind],
    ) -> Result<Box<dyn BackendCounters>, String> {
        let mut processed = 0;

        let mut groups: Vec<PerfCounterGroup> = vec![];
        let mut native_handles: Vec<NativeCounterHandle> = vec![];
        for c in counters {
            // TODO(Alex): for now all events are added together. Instead one should
            // be able to schedule an entire group of events and use multiplexing for
            // each of those groups.
            if (processed > 0) && (processed % 1 == 0) {
                groups.push(PerfCounterGroup::new(native_handles));
                native_handles = vec![];
            }
            let mut attrs = sys::bindings::perf_event_attr::default();
            attrs.size = std::mem::size_of::<sys::bindings::perf_event_attr>() as u32;
            attrs.set_disabled(1);
            attrs.set_exclude_kernel(1);
            attrs.set_exclude_hv(1);
            attrs.read_format = sys::bindings::PERF_FORMAT_GROUP as u64
                | sys::bindings::PERF_FORMAT_ID as u64
                | sys::bindings::PERF_FORMAT_TOTAL_TIME_ENABLED as u64
                | sys::bindings::PERF_FORMAT_TOTAL_TIME_RUNNING as u64;
            attrs.set_inherit(1);

            match c {
                CounterKind::Cycles => {
                    attrs.type_ = sys::bindings::PERF_TYPE_HARDWARE;
                    attrs.config = sys::bindings::PERF_COUNT_HW_CPU_CYCLES as u64;
                }
                CounterKind::Instructions => {
                    attrs.type_ = sys::bindings::PERF_TYPE_HARDWARE;
                    attrs.config = sys::bindings::PERF_COUNT_HW_INSTRUCTIONS as u64;
                }
                CounterKind::Branches => {
                    attrs.type_ = sys::bindings::PERF_TYPE_HARDWARE;
                    attrs.config = sys::bindings::PERF_COUNT_HW_BRANCH_INSTRUCTIONS as u64;
                }
                CounterKind::BranchMisses => {
                    attrs.type_ = sys::bindings::PERF_TYPE_HARDWARE;
                    attrs.config = sys::bindings::PERF_COUNT_HW_BRANCH_MISSES as u64;
                }
                CounterKind::CacheHits => {
                    attrs.type_ = sys::bindings::PERF_TYPE_HARDWARE;
                    attrs.config = sys::bindings::PERF_COUNT_HW_CACHE_REFERENCES as u64;
                }
                CounterKind::CacheMisses => {
                    attrs.type_ = sys::bindings::PERF_TYPE_HARDWARE;
                    attrs.config = sys::bindings::PERF_COUNT_HW_CACHE_MISSES as u64;
                }
                CounterKind::System(counter) => match counter.kind {
                    crate::SystemCounterKind::Software => {
                        attrs.type_ = sys::bindings::PERF_TYPE_SOFTWARE;
                        attrs.config = counter.encoding;
                    }
                    crate::SystemCounterKind::Hardware => {
                        attrs.type_ = sys::bindings::PERF_TYPE_RAW;
                        attrs.config = counter.encoding;
                    }
                },
                _ => {
                    unimplemented!();
                }
            }

            let base_fd: i32 = if native_handles.is_empty() {
                -1
            } else {
                native_handles.first().unwrap().fd
            };

            let new_fd =
                unsafe { sys::perf_event_open(&mut attrs, pid.unwrap_or(0), -1, base_fd, 0) };

            if new_fd < 0 {
                return Err("Failed to open file descriptor".to_string());
            }

            let mut id: u64 = 0;

            let result = unsafe { sys::ioctls::ID(new_fd, &mut id) };
            if result < 0 {
                return Err("Failed to acquire event ID".to_string());
            }

            native_handles.push(NativeCounterHandle {
                kind: c.clone(),
                fd: new_fd,
                id: id,
            });

            processed += 1;
        }

        if !native_handles.is_empty() {
            groups.push(PerfCounterGroup::new(native_handles));
        }

        return Ok(Box::new(PerfCounters::new(groups, pid.unwrap_or(0))));
    }
}

#[cfg(target_os = "linux")]
impl PerfCounters {
    fn new(groups: Vec<PerfCounterGroup>, pid: i32) -> PerfCounters {
        return PerfCounters { groups, pid };
    }
}

#[cfg(target_os = "linux")]
impl PerfCounterGroup {
    fn new(native_handles: Vec<NativeCounterHandle>) -> PerfCounterGroup {
        return PerfCounterGroup {
            native_handles,
            buffer: vec![0; 8192],
        };
    }
}

#[cfg(target_os = "linux")]
impl BackendCounters for PerfCounters {
    fn start(&mut self) {
        for g in &self.groups {
            let res = unsafe {
                sys::ioctls::RESET(
                    g.native_handles.first().unwrap().fd,
                    sys::bindings::PERF_IOC_FLAG_GROUP,
                )
            };
            if res < 0 {
                panic!("Failed to reset counters");
            }
        }
        for g in &self.groups {
            let res_enable = unsafe {
                sys::ioctls::ENABLE(
                    g.native_handles.first().unwrap().fd,
                    sys::bindings::PERF_IOC_FLAG_GROUP,
                )
            };
            if res_enable < 0 {
                panic!("Failed to start profiling");
            }
        }
        if self.pid != 0 {
            let res = unsafe {
                ptrace(
                    libc::PTRACE_CONT,
                    self.pid,
                    std::ptr::null_mut::<libc::c_void>(),
                    std::ptr::null_mut::<libc::c_void>(),
                )
            };
            if res < 0 {
                panic!("Failed to continue the process");
            }
        }
    }
    fn stop(&mut self) {
        for g in &self.groups {
            let res = unsafe {
                sys::ioctls::DISABLE(
                    g.native_handles.first().unwrap().fd,
                    sys::bindings::PERF_IOC_FLAG_GROUP,
                )
            };
            if res < 0 {
                panic!("Failed to reset counters");
            }
        }
        for g in &mut self.groups {
            let res_read = unsafe {
                read(
                    g.native_handles.first().unwrap().fd,
                    g.buffer.as_mut_ptr() as *mut libc::c_void,
                    8192,
                )
            };

            if res_read < 0 {
                panic!("Failed to read output data");
            }
        }
    }

    fn peek(&self, id: usize) -> Option<crate::CounterValue> {
        let group_id = id / 1;
        let event_id = 0;

        if group_id >= self.groups.len() {
            return None;
        }

        // TODO how to unwrap correctly?
        let nr =
            *unsafe { (self.groups[group_id].buffer.as_ptr() as *const u64).as_ref() }.unwrap();
        let time_enabled = *unsafe {
            (self.groups[group_id].buffer.as_ptr() as *const u64)
                .offset(1)
                .as_ref()
        }
        .unwrap();
        let time_running = *unsafe {
            (self.groups[group_id].buffer.as_ptr() as *const u64)
                .offset(2)
                .as_ref()
        }
        .unwrap();

        let div = (time_enabled as f32) / (time_running as f32);

        if event_id >= nr as usize {
            return None;
        }

        let slice = unsafe {
            std::slice::from_raw_parts(
                self.groups[group_id]
                    .buffer
                    .as_ptr()
                    .offset((3 * std::mem::size_of::<u64>()) as isize)
                    as *const RFValues,
                nr as usize,
            )
        };

        let mut cv = crate::CounterValue {
            kind: CounterKind::Cycles,
            value: (slice[event_id].value as f32 / div) as usize,
        };

        // TODO(Alex): use find
        for g in &self.groups {
            for c in &g.native_handles {
                if slice[event_id].id == c.id {
                    cv.kind = c.kind.clone();
                    break;
                }
            }
        }

        return Some(cv);
    }
}

pub(crate) fn get_software_events() -> Vec<crate::SystemCounter> {
    let events = vec![
        SystemCounter {
            kind: SystemCounterKind::Software,
            name: "cpu_clock",
            desc: "A high-resolution per-CPU timer",
            encoding: sys::bindings::PERF_COUNT_SW_CPU_CLOCK as u64,
        },
        SystemCounter {
            kind: SystemCounterKind::Software,
            name: "task_clock",
            desc: "Clock count specific to the task that is running",
            encoding: sys::bindings::PERF_COUNT_SW_TASK_CLOCK as u64,
        },
        SystemCounter {
            kind: SystemCounterKind::Software,
            name: "page_faults",
            desc: "Number of page faults",
            encoding: sys::bindings::PERF_COUNT_SW_PAGE_FAULTS as u64,
        },
        SystemCounter {
            kind: SystemCounterKind::Software,
            name: "context_switches",
            desc: "Number of context switches",
            encoding: sys::bindings::PERF_COUNT_SW_CONTEXT_SWITCHES as u64,
        },
        SystemCounter {
            kind: SystemCounterKind::Software,
            name: "cpu_migrations",
            desc: "Number of times the process has migrated to a new CPU",
            encoding: sys::bindings::PERF_COUNT_SW_CPU_MIGRATIONS as u64,
        },
        SystemCounter {
            kind: SystemCounterKind::Software,
            name: "page_faults_min",
            desc: "Number of minor page faults",
            encoding: sys::bindings::PERF_COUNT_SW_PAGE_FAULTS_MIN as u64,
        },
        SystemCounter {
            kind: SystemCounterKind::Software,
            name: "page_faults_maj",
            desc: "Number of major page faults",
            encoding: sys::bindings::PERF_COUNT_SW_PAGE_FAULTS_MAJ as u64,
        },
        SystemCounter {
            kind: SystemCounterKind::Software,
            name: "alignment_faults",
            desc: "Number of unaligned memory accesses",
            encoding: sys::bindings::PERF_COUNT_SW_ALIGNMENT_FAULTS as u64,
        },
        SystemCounter {
            kind: SystemCounterKind::Software,
            name: "emulation_faults",
            desc: "Number of emulation faults",
            encoding: sys::bindings::PERF_COUNT_SW_EMULATION_FAULTS as u64,
        },
        SystemCounter {
            kind: SystemCounterKind::Software,
            name: "dummy",
            desc: "A placeholder event",
            encoding: sys::bindings::PERF_COUNT_SW_DUMMY as u64,
        },
    ];

    return events;
}
