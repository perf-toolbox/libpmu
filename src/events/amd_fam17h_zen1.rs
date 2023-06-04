use crate::{SystemCounter, SystemCounterKind};

const RETIRED_UOPS: SystemCounter = SystemCounter {
    kind: SystemCounterKind::Hardware,
    name: "RETIRED_UOPS",
    encoding: 0xc1 as u64,
};

const MISALIGNED_LOADS: SystemCounter = SystemCounter {
    kind: SystemCounterKind::Hardware,
    name: "MISALIGNED_LOADS",
    encoding: 0x47 as u64,
};

pub(crate) fn get() -> Vec<crate::SystemCounter> {
    vec![RETIRED_UOPS.clone(), MISALIGNED_LOADS.clone()]
}
