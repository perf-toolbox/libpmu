#[cfg(target_os = "macos")]
use crate::backends::{Backend, BackendCounters};
#[cfg(target_os = "macos")]
use crate::{CounterKind, CountersGroup};
use dlopen2::wrapper::{Container, WrapperApi};
use libc::*;
use std::ffi::CStr;
use std::sync::Arc;

const MAX_COUNTERS: usize = 6;

#[allow(dead_code)]
const KPC_CLASS_FIXED: u32 = 0;
const KPC_CLASS_CONFIGURABLE: u32 = 1;
#[allow(dead_code)]
const KPC_CLASS_POWER: u32 = 2;
#[allow(dead_code)]
const KPC_CLASS_RAWPMU: u32 = 3;

#[allow(dead_code)]
const KPC_CLASS_FIXED_MASK: u32 = 1 << KPC_CLASS_FIXED;
const KPC_CLASS_CONFIGURABLE_MASK: u32 = 1 << KPC_CLASS_CONFIGURABLE;
#[allow(dead_code)]
const KPC_CLASS_POWER_MASK: u32 = 1 << KPC_CLASS_POWER;
#[allow(dead_code)]
const KPC_CLASS_RAWPMU_MASK: u32 = 1 << KPC_CLASS_RAWPMU;

#[repr(C)]
struct KPepEvent {
    name: *const c_char,
    description: *const c_char,
    errata: *const c_char,
    alias: *const c_char,
    fallback: *const c_char,
    mask: u32,
    number: u8,
    umask: u8,
    reserved: u8,
    is_fixed: u8,
}

#[repr(C)]
struct KPepDB {
    name: *const c_char,
    cpu_id: *const c_char,
    marketing_name: *const c_char,
    plist_data: *const u8,
    event_map: *const u8,
    event_arr: *const KPepEvent,
    fixed_event_arr: *const *const KPepEvent,
    alias_map: *const u8,
    reserved1: size_t,
    reserved2: size_t,
    reserved3: size_t,
    event_count: size_t,
    alias_count: size_t,
    fixed_count_counter: size_t,
    config_counter_count: size_t,
    power_counter_count: size_t,
    architecture: u32,
    fixed_counter_bits: u32,
    config_counter_bits: u32,
    power_counter_bits: u32,
}

#[repr(C)]
struct KPepConfig {
    db: *const KPepDB,
    ev_arr: *const *const KPepEvent,
    ev_map: *const size_t,
    ev_idx: *const size_t,
    flags: *const u32,
    kpc_periods: *const u64,
    event_count: size_t,
    counter_count: size_t,
    classes: u32,
    config_counter: u32,
    power_counter: u32,
    reserved: u32,
}

#[derive(WrapperApi)]
struct KPCDispatch {
    kpc_cpu_string: unsafe extern "C" fn(buf: *mut c_char, buf_size: size_t) -> c_int,
    kpc_pmu_version: unsafe extern "C" fn() -> u32,
    kpc_set_counting: unsafe extern "C" fn(classes: u32) -> c_int,
    kpc_set_thread_counting: unsafe extern "C" fn(classes: u32) -> c_int,
    kpc_set_config: unsafe extern "C" fn(classes: u32, config: *mut u64) -> c_int,
    kpc_get_thread_counters: unsafe extern "C" fn(tid: u32, buf_count: u32, buf: *mut u64) -> c_int,
    kpc_force_all_ctrs_set: unsafe extern "C" fn(val: c_int) -> c_int,
}

#[derive(WrapperApi)]
struct KPEPDispatch {
    kpep_config_create: unsafe extern "C" fn(db: *mut KPepDB, cfg: *mut *mut KPepConfig) -> c_int,
    kpep_config_free: unsafe extern "C" fn(cfg: *mut KPepConfig),
    kpep_db_create: unsafe extern "C" fn(name: *const c_char, db: *mut *mut KPepDB) -> c_int,
    kpep_db_free: unsafe extern "C" fn(db: *mut KPepDB),
    kpep_db_events_count: unsafe extern "C" fn(db: *const KPepDB, count: *mut size_t) -> c_int,
    kpep_db_events:
        unsafe extern "C" fn(db: *const KPepDB, buf: *mut *mut KPepEvent, buf_size: usize) -> c_int,
    kpep_event_name: unsafe extern "C" fn(event: *const KPepEvent, name: *mut *const c_char),
    kpep_event_description: unsafe extern "C" fn(event: *const KPepEvent, desc: *mut *const c_char),
    kpep_config_force_counters: unsafe extern "C" fn(cfg: *mut KPepConfig) -> c_int,
    kpep_config_add_event: unsafe extern "C" fn(
        cfg: *mut KPepConfig,
        evt: *mut *mut KPepEvent,
        flag: u32,
        err: *mut u32,
    ) -> c_int,
    kpep_config_kpc:
        unsafe extern "C" fn(cfg: *mut KPepConfig, buf: *mut u64, buf_size: usize) -> c_int,
    kpep_config_kpc_count:
        unsafe extern "C" fn(cfg: *mut KPepConfig, count_ptr: *mut usize) -> c_int,
    kpep_config_kpc_classes:
        unsafe extern "C" fn(cfg: *mut KPepConfig, classes_ptr: *mut u32) -> c_int,
    kpep_config_kpc_map:
        unsafe extern "C" fn(cfg: *mut KPepConfig, buf: *mut usize, size: usize) -> c_int,
}

#[cfg(target_os = "macos")]
struct NativeCounterHandle {
    pub kind: CounterKind,
    pub reg_id: usize,
}

#[cfg(target_os = "macos")]
struct KPerfCounters {
    kpc_dispatch: Arc<Container<KPCDispatch>>,
    kpep_dispatch: Arc<Container<KPEPDispatch>>,
    native_handles: Vec<NativeCounterHandle>,
    counter_values_before: Vec<u64>,
    counter_values_after: Vec<u64>,
    config: *mut KPepConfig,
}

#[cfg(target_os = "macos")]
pub struct KPerfBackend {
    kpc_dispatch: Arc<Container<KPCDispatch>>,
    kpep_dispatch: Arc<Container<KPEPDispatch>>,
    db: *const KPepDB,
}

#[cfg(target_os = "macos")]
impl KPerfBackend {
    pub fn new() -> KPerfBackend {
        let kpc_dispatch: Container<KPCDispatch> =
            unsafe { Container::load("/System/Library/PrivateFrameworks/kperf.framework/kperf") }
                .expect("Could not open library or load symbols");
        let kpep_dispatch: Container<KPEPDispatch> = unsafe {
            Container::load("/System/Library/PrivateFrameworks/kperfdata.framework/kperfdata")
        }
        .expect("Could not open library or load symbols");

        let mut db: *mut KPepDB = std::ptr::null_mut();
        if unsafe { kpep_dispatch.kpep_db_create(std::ptr::null(), &mut db) } != 0 {
            panic!("Failed to load kpep database");
        }

        return KPerfBackend {
            kpc_dispatch: kpc_dispatch.into(),
            kpep_dispatch: kpep_dispatch.into(),
            db,
        };
    }
}

fn event_matches_name(e: *mut KPepEvent, name: &str) -> bool {
    let c_str: &CStr = unsafe { CStr::from_ptr((*e).name) };
    let str_slice: &str = c_str.to_str().unwrap();

    return str_slice == name;
}

macro_rules! macos_event {
    ($m1_name:expr, $intel_name:expr, $kperf_events:ident, $cfg:ident, $dispatch:expr) => {
        let m1_event = $kperf_events
            .iter()
            .find(|e| event_matches_name(*(*e), $m1_name));
        let intel_event = $kperf_events
            .iter()
            .find(|e| event_matches_name(*(*e), $intel_name));

        let mut event: *mut KPepEvent = m1_event.or(intel_event).unwrap().clone();

        if unsafe {
            $dispatch.kpep_config_add_event($cfg, &mut event, 0, std::ptr::null_mut()) != 0
        } {
            panic!("Failed to add an event");
        }
    };
}

#[cfg(target_os = "macos")]
impl Backend for KPerfBackend {
    fn create_counters(
        &self,
        _pid: Option<i32>,
        groups: &[CountersGroup],
    ) -> Result<Box<dyn BackendCounters>, String> {
        if groups.len() != 1 {
            return Err(format!("Only 1 group is supported currently"));
        }
        if groups.first().unwrap().counters.len() > MAX_COUNTERS {
            return Err(format!(
                "Only {} counters are supported right now",
                MAX_COUNTERS
            ));
        }
        let mut num_events: size_t = 0;
        if unsafe {
            self.kpep_dispatch
                .kpep_db_events_count(self.db, &mut num_events)
        } != 0
        {
            return Err("Failed to count events".to_string());
        }

        let mut kperf_events: Vec<*mut KPepEvent> = Vec::with_capacity(num_events as usize);
        kperf_events.resize(num_events, std::ptr::null_mut());
        if unsafe {
            self.kpep_dispatch.kpep_db_events(
                self.db,
                kperf_events.as_mut_ptr(),
                num_events * std::mem::size_of::<*mut u8>(),
            )
        } != 0
        {
            return Err("Failed to query events".to_string());
        }

        let mut db: *mut KPepDB = std::ptr::null_mut();
        if unsafe { self.kpep_dispatch.kpep_db_create(std::ptr::null(), &mut db) } != 0 {
            panic!("Failed to load kpep database");
        }

        let mut cfg: *mut KPepConfig = std::ptr::null_mut();
        if unsafe { self.kpep_dispatch.kpep_config_create(db, &mut cfg) } != 0 {
            panic!("Failed to create config");
        }
        if unsafe { self.kpep_dispatch.kpep_config_force_counters(cfg) != 0 } {
            panic!("Failed to set counters");
        }

        let mut native_handles = vec![];

        for c in &groups.first().unwrap().counters {
            native_handles.push(NativeCounterHandle {
                kind: c.counter.clone(),
                reg_id: 0,
            });
            match c.counter {
                CounterKind::Cycles => {
                    macos_event!(
                        "FIXED_CYCLES",
                        "CPU_CLK_UNHALTED.THREAD",
                        kperf_events,
                        cfg,
                        self.kpep_dispatch
                    );
                }
                CounterKind::Instructions => {
                    macos_event!(
                        "FIXED_INSTRUCTIONS",
                        "INST_RETIRED.ANY",
                        kperf_events,
                        cfg,
                        self.kpep_dispatch
                    );
                }
                CounterKind::Branches => {
                    macos_event!(
                        "INST_BRANCH",
                        "BR_INST_RETIRED.ALL_BRANCHES",
                        kperf_events,
                        cfg,
                        self.kpep_dispatch
                    );
                }
                CounterKind::BranchMisses => {
                    macos_event!(
                        "INST_BRANCH",
                        "BR_MISP_RETIRED.ALL_BRANCHES",
                        kperf_events,
                        cfg,
                        self.kpep_dispatch
                    );
                }
                _ => {}
            }
        }

        return Ok(Box::new(KPerfCounters {
            kpc_dispatch: self.kpc_dispatch.clone(),
            kpep_dispatch: self.kpep_dispatch.clone(),
            native_handles,
            counter_values_before: vec![0; 32],
            counter_values_after: vec![0; 32],
            config: cfg.clone(),
        }));
    }
}

#[cfg(target_os = "macos")]
impl BackendCounters for KPerfCounters {
    fn start(&mut self) {
        let mut classes: u32 = 0;
        if unsafe {
            self.kpep_dispatch
                .kpep_config_kpc_classes(self.config, &mut classes)
                != 0
        } {
            panic!("Failed to get kpc classes");
        }

        let mut reg_count: usize = 0;
        if unsafe {
            self.kpep_dispatch
                .kpep_config_kpc_count(self.config, &mut reg_count)
                != 0
        } {
            panic!("Failed to get kpc count");
        }

        // TODO(Alex): 32 is a hardcore value here
        let mut native_reg_map = vec![];
        native_reg_map.resize(32, 0);
        let ret_val = unsafe {
            self.kpep_dispatch.kpep_config_kpc_map(
                self.config,
                native_reg_map.as_mut_ptr(),
                native_reg_map.len() * std::mem::size_of::<usize>(),
            )
        };
        if ret_val != 0 {
            panic!("Failed to get register mapping");
        }

        for i in 0..self.native_handles.len() {
            self.native_handles[i].reg_id = native_reg_map[i];
        }

        let mut regs = vec![];
        regs.resize(reg_count, 0);
        println!("Reg count is {}", reg_count);
        if unsafe {
            self.kpep_dispatch.kpep_config_kpc(
                self.config,
                regs.as_mut_ptr(),
                reg_count * std::mem::size_of::<u64>(),
            ) != 0
        } {
            panic!("Failed to set kpc config");
        }

        if unsafe { self.kpc_dispatch.kpc_force_all_ctrs_set(1) != 0 } {
            panic!("Failed to set kpc counters");
        }

        if (classes & KPC_CLASS_CONFIGURABLE_MASK != 0) && reg_count != 0 {
            if unsafe { self.kpc_dispatch.kpc_set_config(classes, regs.as_mut_ptr()) != 0 } {
                panic!("Failed to set kpc config");
            }
        }

        if unsafe { self.kpc_dispatch.kpc_set_counting(classes) != 0 } {
            panic!("Failed to set counting");
        }
        if unsafe { self.kpc_dispatch.kpc_set_thread_counting(classes) != 0 } {
            panic!("Failet to set thread counting");
        }

        if unsafe {
            self.kpc_dispatch.kpc_get_thread_counters(
                0,
                32,
                self.counter_values_before.as_mut_ptr(),
            ) != 0
        } {
            panic!("Failed to get counters");
        }
    }
    fn stop(&mut self) {
        if unsafe {
            self.kpc_dispatch
                .kpc_get_thread_counters(0, 32, self.counter_values_after.as_mut_ptr())
                != 0
        } {
            panic!("Failed to get counters");
        }
        unsafe {
            self.kpc_dispatch.kpc_set_counting(0);
            self.kpc_dispatch.kpc_set_thread_counting(0);
        }
    }

    fn peek(&self, id: usize) -> Option<crate::CounterValue> {
        if id >= self.native_handles.len() {
            return None;
        }

        let reg_id = self.native_handles[id].reg_id;
        return Some(crate::CounterValue {
            kind: self.native_handles[id].kind.clone(),
            value: (self.counter_values_after[reg_id] - self.counter_values_before[reg_id])
                as usize,
        });
    }
}

pub(crate) fn get_software_events() -> Vec<crate::SystemCounter> {
    vec![]
}
