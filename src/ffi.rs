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

    if kind == PMUCounterKind_PMU_CYCLES {
        builder.builder.add_counter(crate::CounterKind::Cycles);
        return 0;
    } else if kind == PMUCounterKind_PMU_INSTRUCTIONS {
        builder
            .builder
            .add_counter(crate::CounterKind::Instructions);
        return 0;
    } else if kind == PMUCounterKind_PMU_BRANCHES {
        builder.builder.add_counter(crate::CounterKind::Branches);
        return 0;
    } else if kind == PMUCounterKind_PMU_BRANCH_MISSES {
        builder
            .builder
            .add_counter(crate::CounterKind::BranchMisses);
        return 0;
    } else if kind == PMUCounterKind_PMU_SYSTEM {
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
pub extern "C" fn pmu_builder_add_cache_counter(
    builder_raw: *mut FFIBuilder,
    clevel: c_int,
    ckind: c_int,
    cop: c_int,
    name_raw: *const c_char,
) -> c_int {
    if builder_raw == std::ptr::null_mut() {
        return 1;
    }

    let builder = unsafe { builder_raw.as_mut() }.unwrap();

    let level = if clevel == PMUCacheLevelKind_PMU_CACHE_L1 {
        Some(crate::CacheLevelKind::L1)
    } else if clevel == PMUCacheLevelKind_PMU_CACHE_L1D {
        Some(crate::CacheLevelKind::L1D)
    } else if clevel == PMUCacheLevelKind_PMU_CACHE_L1I {
        Some(crate::CacheLevelKind::L1I)
    } else if clevel == PMUCacheLevelKind_PMU_CACHE_L2 {
        Some(crate::CacheLevelKind::L2)
    } else if clevel == PMUCacheLevelKind_PMU_CACHE_L3 {
        Some(crate::CacheLevelKind::L3)
    } else if clevel == PMUCacheLevelKind_PMU_CACHE_LAST {
        Some(crate::CacheLevelKind::Last)
    } else if clevel == PMUCacheLevelKind_PMU_CACHE_DTLB {
        Some(crate::CacheLevelKind::DTLB)
    } else if clevel == PMUCacheLevelKind_PMU_CACHE_ITLB {
        Some(crate::CacheLevelKind::ITLB)
    } else if clevel == PMUCacheLevelKind_PMU_CACHE_BPU {
        Some(crate::CacheLevelKind::BPU)
    } else {
        None
    };

    let kind = if ckind == PMUCacheCounterKind_PMU_CACHE_HIT {
        Some(crate::CacheCounterKind::Hit)
    } else if ckind == PMUCacheCounterKind_PMU_CACHE_MISS {
        Some(crate::CacheCounterKind::Miss)
    } else {
        None
    };

    let op = if cop == PMUCacheOpKind_PMU_CACHE_READ {
        Some(crate::CacheOpKind::Read)
    } else if ckind == PMUCacheOpKind_PMU_CACHE_WRITE {
        Some(crate::CacheOpKind::Write)
    } else if ckind == PMUCacheOpKind_PMU_CACHE_PREFETCH {
        Some(crate::CacheOpKind::Prefetch)
    } else {
        None
    };

    if level.is_none() || kind.is_none() || op.is_none() {
        return 1;
    }


    builder
        .builder
        .add_counter(crate::CounterKind::Cache(crate::CacheCounter{kind: kind.unwrap(), level: level.unwrap(), op: op.unwrap()}));
    return 0;
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
