#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use libc::{c_char, c_int};
use std::ffi::CStr;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub struct FFIBuilder {
    builder: crate::Builder,
}

pub struct FFICounters {
    counters: crate::Counters,
}

#[no_mangle]
pub extern "C" fn pmu_builder_create() -> *mut FFIBuilder {
    let builder = crate::Builder::new();

    let managed_builder = Box::new(FFIBuilder { builder });
    return Box::leak(managed_builder);
}

#[no_mangle]
pub extern "C" fn pmu_builder_release(builder: *mut FFIBuilder) {
    if builder != std::ptr::null_mut() {
        unsafe {
            drop(Box::from_raw(builder));
        }
    }
}

#[no_mangle]
pub extern "C" fn pmu_builder_add_counter(
    builder_raw: *mut FFIBuilder,
    kind: c_int,
    name_raw: *const c_char,
) -> c_int {
    if builder_raw == std::ptr::null_mut() {
        return 1;
    }

    let builder = unsafe { builder_raw.as_mut() }.unwrap();

    if kind == PMUCounterKind_CYCLES {
        builder.builder.add_counter(crate::CounterKind::Cycles);
        return 0;
    } else if kind == PMUCounterKind_INSTRUCTIONS {
        builder
            .builder
            .add_counter(crate::CounterKind::Instructions);
        return 0;
    } else if kind == PMUCounterKind_CACHE_MISSES {
        builder.builder.add_counter(crate::CounterKind::CacheMisses);
        return 0;
    } else if kind == PMUCounterKind_BRANCHES {
        builder.builder.add_counter(crate::CounterKind::Branches);
        return 0;
    } else if kind == PMUCounterKind_BRANCH_MISSES {
        builder
            .builder
            .add_counter(crate::CounterKind::BranchMisses);
        return 0;
    } else if kind == PMUCounterKind_SYSTEM {
        if name_raw == std::ptr::null() {
            return 1;
        }

        let name: &CStr = unsafe { CStr::from_ptr(name_raw) };
        let event = crate::find_event_by_name(name.to_str().unwrap());

        match event {
            Some(event) => {
                builder
                    .builder
                    .add_counter(crate::CounterKind::System(event));
                return 0;
            }
            None => {
                return 1;
            }
        }
    }

    return 1;
}

#[no_mangle]
pub extern "C" fn pmu_builder_build(builder_raw: *mut FFIBuilder) -> *mut FFICounters {
    if builder_raw == std::ptr::null_mut() {
        return std::ptr::null_mut();
    }

    let builder = unsafe { builder_raw.as_mut() }.unwrap();

    let counters = builder.builder.build();

    match counters {
        Ok(counters) => {
            let counters_managed = Box::new(FFICounters { counters });

            return Box::leak(counters_managed);
        }
        Err(_) => {
            return std::ptr::null_mut();
        }
    }
}

#[no_mangle]
pub extern "C" fn pmu_counters_start(counters_raw: *mut FFICounters) {
    if counters_raw == std::ptr::null_mut() {
        return;
    }

    let counters = unsafe { counters_raw.as_mut() }.unwrap();
    counters.counters.start()
}

#[no_mangle]
pub extern "C" fn pmu_counters_stop(counters_raw: *mut FFICounters) {
    if counters_raw == std::ptr::null_mut() {
        return;
    }

    let counters = unsafe { counters_raw.as_mut() }.unwrap();
    counters.counters.stop()
}

#[no_mangle]
pub extern "C" fn pmu_counters_peek_name(
    counters_raw: *mut FFICounters,
    id: c_int,
    len_raw: *mut usize,
    str_raw: *mut u8,
) -> c_int {
    if counters_raw == std::ptr::null_mut() {
        return 1;
    }
    if len_raw == std::ptr::null_mut() && str_raw == std::ptr::null_mut() {
        return 1;
    }

    let counters = unsafe { counters_raw.as_mut() }.unwrap();

    let result = counters.counters.backend_counters.peek(id as usize);

    match result {
        Some(result) => {
            if len_raw != std::ptr::null_mut() {
                let len = unsafe { len_raw.as_mut() }.unwrap();
                *len = result.kind.to_string().len() + 1;
                return 0;
            } else {
                let name = result.kind.to_string();
                let name_parts = unsafe { std::slice::from_raw_parts_mut(str_raw, name.len()) };
                name_parts.clone_from_slice(name.as_bytes());
                return 0;
            }
        }
        None => {
            return 1;
        }
    }
}

#[no_mangle]
pub extern "C" fn pmu_counters_peek_value(
    counters_raw: *mut FFICounters,
    id: c_int,
    value_raw: *mut u64,
) -> c_int {
    if counters_raw == std::ptr::null_mut() {
        return 1;
    }
    if value_raw == std::ptr::null_mut() {
        return 1;
    }

    let counters = unsafe { counters_raw.as_mut() }.unwrap();
    let value = unsafe { value_raw.as_mut() }.unwrap();

    let result = counters.counters.backend_counters.peek(id as usize);

    match result {
        Some(result) => {
            *value = result.value as u64;
            return 0;
        }
        None => {
            return 1;
        }
    }
}
