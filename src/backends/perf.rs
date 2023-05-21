use crate::backends::{Backend, BackendCounters};
use libc::read;
use perf_event_open_sys as sys;

pub(crate) struct PerfBackend {}

#[repr(C)]
struct RFValues {
    value: u64,
    id: u64,
}

struct NativeCounterHandle {
    pub kind: crate::CounterType,
    pub fd: i32,
    pub id: u64,
}

struct PerfCounters {
    native_handles: Vec<NativeCounterHandle>,
    buffer: Vec<u8>,
}

impl PerfBackend {
    pub fn new() -> PerfBackend {
        return PerfBackend {};
    }
}

impl Backend for PerfBackend {
    fn create_counters(
        &self,
        _pid: Option<i32>,
        _period: u32,
        counters: &[crate::CounterType],
    ) -> Result<Box<dyn BackendCounters>, String> {
        let mut native_handles: Vec<NativeCounterHandle> = vec![];

        for c in counters {
            let mut attrs = sys::bindings::perf_event_attr::default();
            attrs.size = std::mem::size_of::<sys::bindings::perf_event_attr>() as u32;
            attrs.set_disabled(1);
            attrs.set_exclude_kernel(1);
            attrs.set_exclude_hv(1);
            attrs.read_format =
                sys::bindings::PERF_FORMAT_GROUP as u64 | sys::bindings::PERF_FORMAT_ID as u64;

            match c {
                crate::CounterType::Cycles => {
                    attrs.type_ = sys::bindings::PERF_TYPE_HARDWARE;
                    attrs.config = sys::bindings::PERF_COUNT_HW_CPU_CYCLES as u64;
                }
                crate::CounterType::Instructions => {
                    attrs.type_ = sys::bindings::PERF_TYPE_HARDWARE;
                    attrs.config = sys::bindings::PERF_COUNT_HW_INSTRUCTIONS as u64;
                }
                crate::CounterType::Branches => {
                    attrs.type_ = sys::bindings::PERF_TYPE_HARDWARE;
                    attrs.config = sys::bindings::PERF_COUNT_HW_BRANCH_INSTRUCTIONS as u64;
                }
                crate::CounterType::BranchMisses => {
                    attrs.type_ = sys::bindings::PERF_TYPE_HARDWARE;
                    attrs.config = sys::bindings::PERF_COUNT_HW_BRANCH_MISSES as u64;
                }
                _ => {
                    unimplemented!();
                }
            }

            let base_fd: i32 = if native_handles.is_empty() {
                -1
            } else {
                native_handles.first().unwrap().fd
            };

            let new_fd = unsafe { sys::perf_event_open(&mut attrs, 0, -1, base_fd, 0) };

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
        }

        return Ok(Box::new(PerfCounters::new(native_handles)));
    }
}

impl PerfCounters {
    fn new(native_handles: Vec<NativeCounterHandle>) -> PerfCounters {
        return PerfCounters {
            native_handles,
            buffer: vec![0; 8192],
        };
    }
}

impl BackendCounters for PerfCounters {
    fn start(&mut self) {
        let res = unsafe {
            sys::ioctls::RESET(
                self.native_handles.first().unwrap().fd,
                sys::bindings::PERF_IOC_FLAG_GROUP,
            )
        };
        if res < 0 {
            panic!("Failed to reset counters");
        }
        let res_enable = unsafe {
            sys::ioctls::ENABLE(
                self.native_handles.first().unwrap().fd,
                sys::bindings::PERF_IOC_FLAG_GROUP,
            )
        };
        if res_enable < 0 {
            panic!("Failed to start profiling");
        }
    }
    fn stop(&mut self) {
        let res = unsafe {
            sys::ioctls::DISABLE(
                self.native_handles.first().unwrap().fd,
                sys::bindings::PERF_IOC_FLAG_GROUP,
            )
        };
        if res < 0 {
            panic!("Failed to reset counters");
        }

        let res_read = unsafe {
            read(
                self.native_handles.first().unwrap().fd,
                self.buffer.as_mut_ptr() as *mut libc::c_void,
                8192,
            )
        };

        if res_read < 0 {
            panic!("Failed to read output data");
        }
    }

    fn peek(&self, id: usize) -> Option<crate::CounterValue> {
        // TODO how to unwrap correctly?
        let nr = unsafe { (self.buffer.as_ptr() as *const u64).as_ref() }.unwrap();

        if id >= *nr as usize {
            return None;
        }

        let slice = unsafe {
            std::slice::from_raw_parts(
                (self.buffer.as_ptr() as *const u64).offset(1) as *const RFValues,
                *nr as usize,
            )
        };

        let mut cv = crate::CounterValue {
            kind: crate::CounterType::Cycles,
            value: slice[id].value as usize,
        };

        // TODO(Alex): use find
        for c in &self.native_handles {
            if slice[id].id == c.id {
                cv.kind = c.kind.clone();
                break;
            }
        }

        return Some(cv);
    }
}
