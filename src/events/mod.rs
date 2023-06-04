use crate::SystemCounter;

#[cfg(target_arch = "x86_64")]
mod amd_fam17h_zen1;
#[cfg(target_arch = "x86_64")]
mod intel_icl;

#[cfg(target_arch = "x86_64")]
#[repr(C)]
struct X86VendorInfo {
    ebx: u32,
    edx: u32,
    ecx: u32,
}

#[cfg(target_arch = "x86_64")]
impl X86VendorInfo {
    fn new(result: core::arch::x86_64::CpuidResult) -> X86VendorInfo {
        X86VendorInfo {
            ebx: result.ebx,
            ecx: result.ecx,
            edx: result.edx,
        }
    }

    fn as_str<'a>(&'a self) -> &'a str {
        use std::mem::size_of;

        let vendor_ptr = self as *const X86VendorInfo as *const u8;
        let slice = unsafe { std::slice::from_raw_parts(vendor_ptr, size_of::<X86VendorInfo>()) };

        std::str::from_utf8(slice).unwrap_or("Invalid")
    }
}

#[cfg(target_arch = "x86_64")]
fn get_x86_64_hardware_events() -> Vec<SystemCounter> {
    const EAX_VENDOR_INFO: u32 = 0x0;

    let vendor_result = unsafe { core::arch::x86_64::__cpuid(EAX_VENDOR_INFO) };
    let vendor_info = X86VendorInfo::new(vendor_result);

    if vendor_info.as_str() == "AuthenticAMD" {
        return amd_fam17h_zen1::get();
    } else if vendor_info.as_str() == "GenuineIntel" {
        return intel_icl::get();
    }

    println!("{}", vendor_info.as_str());
    unreachable!("Unsupported vendor")
}

pub fn get_hardware_events() -> Vec<SystemCounter> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch="x86_64")] {
            return get_x86_64_hardware_events();
        } else {
            return vec![];
            // unimplemented!()
        }
    }
}
