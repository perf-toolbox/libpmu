#ifndef LIB_PMU_ENUMS_H
#define LIB_PMU_ENUMS_H

enum PMUCounterKind : int {
  PMU_CYCLES = 0,
  PMU_INSTRUCTIONS = 1,
  // CACHE_MISSES = 2, <- removed
  PMU_BRANCHES = 3,
  PMU_BRANCH_MISSES = 4,
  PMU_SYSTEM = 5,
  PMU_CACHE = 6,
};

enum PMUCacheLevelKind : int {
  PMU_CACHE_L1 = 0,
  PMU_CACHE_L1I = 1,
  PMU_CACHE_L1D = 2,
  PMU_CACHE_L2 = 3,
  PMU_CACHE_L3 = 4,
  PMU_CACHE_LAST = 5,
  PMU_CACHE_DTLB = 6,
  PMU_CACHE_ITLB = 7,
  PMU_CACHE_BPU = 8,
};

enum PMUCacheCounterKind : int {
  PMU_CACHE_HIT = 0,
  PMU_CACHE_MISS = 1,
};

enum PMUCacheOpKind : int {
  PMU_CACHE_READ = 0,
  PMU_CACHE_WRITE = 1,
  PMU_CACHE_PREFETCH = 2,
};
#endif // LIB_PMU_ENUMS_H
