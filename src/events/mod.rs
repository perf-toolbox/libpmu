use crate::{SystemCounter, SystemCounterKind};

include!(concat!(env!("OUT_DIR"), "/archs.rs"));

#[cfg(target_arch = "x86_64")]
#[repr(C)]
struct X86VendorInfo {
    eax: u32,
}

#[allow(dead_code)]
pub enum ProcessorFamily {
    Unknown,
    AmdZen1,
    AmdZen2,
    AmdZen3,
    AmdZen4,
    IntelBroadwell,
    IntelHaswell,
    IntelSkylake,
    IntelKabyLake,
    IntelCometLake,
    IntelIceLakeClient,
    IntelIceLakeServer,
    IntelTigerLake,
    IntelRocketLake,
    IntelAlderLake,
    IntelRaptorLake,
    SiFiveU7,
}

#[cfg(target_arch = "x86_64")]
impl X86VendorInfo {
    fn new(result: core::arch::x86_64::CpuidResult) -> X86VendorInfo {
        X86VendorInfo { eax: result.eax }
    }

    fn as_host_arch(&self) -> ProcessorFamily {
        let model = (self.eax >> 4) & 0xf;
        let family = (self.eax >> 8) & 0xf;
        let extended_model = (self.eax >> 16) & 0xf;
        let extended_family = (self.eax >> 20) & 0xff;

        if family == 0xf && extended_family == 0x8 {
            // AMD Family 23 (17h)
            if extended_model == 0x0 || extended_model == 0x1 || extended_model == 0x2 {
                return ProcessorFamily::AmdZen1;
            } else if extended_model == 0x3
                || extended_model == 0x4
                || extended_model == 0x6
                || extended_model == 0x7
                || extended_model == 0x9
            {
                return ProcessorFamily::AmdZen2;
            }
        } else if family == 0xf && extended_family == 0xa {
            // AMD Family 25 (19h)
            if extended_model == 0x0
                || extended_model == 0x2
                || extended_model == 0x4
                || extended_model == 0x5
            {
                return ProcessorFamily::AmdZen3;
            } else if extended_model == 0x1 || extended_model == 0x6 || extended_model == 0x7 {
                return ProcessorFamily::AmdZen4;
            }
        } else if family == 0x6 && extended_family == 0 {
            // Recent Intel processors
            if (extended_model == 0x3 && model == 0xc)
                || (extended_model == 0x4 && (model == 0x5 || model == 0x6))
            {
                return ProcessorFamily::IntelHaswell;
            } else if (extended_model == 0x3 && model == 0xd)
                || (extended_model == 0x4 && model == 0x7)
            {
                return ProcessorFamily::IntelBroadwell;
            } else if (extended_model == 0x5 && model == 0xe)
                || (extended_model == 0x4 && model == 0xe)
            {
                return ProcessorFamily::IntelSkylake;
            } else if (extended_model == 0x8 && model == 0xe)
                || (extended_model == 0x9 && model == 0xe)
            {
                return ProcessorFamily::IntelKabyLake;
            } else if extended_model == 0xa && model == 0x5 {
                return ProcessorFamily::IntelCometLake;
            } else if extended_model == 0x7 && model == 0xe {
                return ProcessorFamily::IntelIceLakeClient;
            } else if extended_model == 0x6 && (model == 0xc || model == 0xa) {
                return ProcessorFamily::IntelIceLakeServer;
            } else if extended_model == 0x8 && (model == 0xc || model == 0xd) {
                return ProcessorFamily::IntelTigerLake;
            } else if extended_model == 0xa && model == 0x7 {
                return ProcessorFamily::IntelRocketLake;
            } else if extended_model == 0x9 && (model == 0x7 || model == 0xa) {
                return ProcessorFamily::IntelAlderLake;
            } else if extended_model == 0xb && (model == 0x7 || model == 0xa) {
                return ProcessorFamily::IntelRaptorLake;
            }
        }

        return ProcessorFamily::Unknown;
    }
}

#[cfg(target_arch = "x86_64")]
fn get_x86_64_family() -> ProcessorFamily {
    const EAX_VENDOR_INFO: u32 = 0x1;

    let vendor_result = unsafe { core::arch::x86_64::__cpuid(EAX_VENDOR_INFO) };
    let vendor_info = X86VendorInfo::new(vendor_result);

    return vendor_info.as_host_arch();
}

#[cfg(target_arch = "riscv64")]
fn get_riscv64_family() -> ProcessorFamily {
    use proc_getter::cpuinfo::cpuinfo;

    let info = cpuinfo().expect("/proc/cpuinfo is inaccessible");
    let marchid = info[0].get("marchid");

    match marchid {
        Some(marchid) => {
            if marchid == "0x8000000000000007" {
                // TODO(Alex): technically speaking this also includes E7 and S7
                ProcessorFamily::SiFiveU7
            } else {
                ProcessorFamily::Unknown
            }
        }
        None => ProcessorFamily::Unknown,
    }
}

pub fn get_processor_family() -> ProcessorFamily {
    cfg_if::cfg_if! {
        if #[cfg(target_arch="x86_64")] {
            get_x86_64_family()
        } else if #[cfg(target_arch="riscv64")] {
            get_riscv64_family()
        } else {
            ProcessorFamily::Unknown
        }
    }
}

pub fn get_hardware_events() -> Vec<SystemCounter> {
    let arch = get_processor_family();

    match arch {
        ProcessorFamily::AmdZen1 => amd_fam17h_zen1::get(),
        ProcessorFamily::IntelTigerLake => intel_icl::get(),
        ProcessorFamily::SiFiveU7 => sifive_u7::get(),
        _ => vec![],
    }
}
