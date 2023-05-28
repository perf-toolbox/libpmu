#ifndef LIB_PMU_ENUMS_H
#define LIB_PMU_ENUMS_H

enum PMUCounterKind : int {
  CYCLES = 0,
  INSTRUCTIONS = 1,
  CACHE_MISSES = 2,
  BRANCHES = 3,
  BRANCH_MISSES = 4,
  SYSTEM = 5,
};

#endif // LIB_PMU_ENUMS_H
