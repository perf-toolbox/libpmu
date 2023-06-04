use crate::{SystemCounter, SystemCounterKind};

const INTEL_X86_INV_BIT: u32 = 23;
const INTEL_X86_CMASK_BIT: u32 = 23;

const UOPS_RETIRED_SLOTS: SystemCounter = SystemCounter {
    kind: SystemCounterKind::Hardware,
    name: "UOPS_RETIRED.SLOTS",
    encoding: (0xc2 as u64) | (0x0200 as u64),
};

const UOPS_RETIRED_TOTAL_CYCLES: SystemCounter = SystemCounter {
    kind: SystemCounterKind::Hardware,
    name: "UOPS_RETIRED.TOTAL_CYCLES",
    encoding: (0xc2 as u64)
        | (0x0200 as u64)
        | ((1 as u64) << INTEL_X86_INV_BIT)
        | ((0xa as u64) << INTEL_X86_CMASK_BIT),
};

const UOPS_RETIRED_STALL_CYCLES: SystemCounter = SystemCounter {
    kind: SystemCounterKind::Hardware,
    name: "UOPS_RETIRED.STALL_CYCLES",
    encoding: (0xc2 as u64)
        | (0x0200 as u64)
        | ((1 as u64) << INTEL_X86_INV_BIT)
        | ((1 as u64) << INTEL_X86_CMASK_BIT),
};

const MEM_INST_RETIRED_ALL_STORES: SystemCounter = SystemCounter {
    kind: SystemCounterKind::Hardware,
    name: "MEM_INST_RETIRED.ALL_STORES",
    encoding: (0xd0 as u64) | (0x8200 as u64),
};

const MEM_INST_RETIRED_ALL_LOADS: SystemCounter = SystemCounter {
    kind: SystemCounterKind::Hardware,
    name: "MEM_INST_RETIRED.ALL_STORES",
    encoding: (0xd0 as u64) | (0x8100 as u64),
};

const MEM_INST_RETIRED_SPLIT_STORES: SystemCounter = SystemCounter {
    kind: SystemCounterKind::Hardware,
    name: "MEM_INST_RETIRED.SPLIT_STORES",
    encoding: (0xd0 as u64) | (0x4200 as u64),
};

const MEM_INST_RETIRED_SPLIT_LOADS: SystemCounter = SystemCounter {
    kind: SystemCounterKind::Hardware,
    name: "MEM_INST_RETIRED.SPLIT_STORES",
    encoding: (0xd0 as u64) | (0x4100 as u64),
};

pub(crate) fn get() -> Vec<crate::SystemCounter> {
    vec![
        UOPS_RETIRED_SLOTS.clone(),
        UOPS_RETIRED_TOTAL_CYCLES.clone(),
        UOPS_RETIRED_STALL_CYCLES.clone(),
    ]
}
