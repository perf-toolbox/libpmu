#ifndef LIB_PMU
#define LIB_PMU

#include "pmu/pmu_enums.h"

#include <cstddef>
#include <cstdint>

#if __cplusplus
extern "C" {
#endif

struct PMUBuilderHandle;
struct PMUCountersHandle;

struct PMUBuilderHandle *pmu_builder_create();
void pmu_builder_release(struct PMUBuilderHandle *);
int pmu_builder_add_counter(struct PMUBuilderHandle *, PMUCounterKind,
                            const char *);
struct PMUCountersHandle *pmu_builder_build(struct PMUBuilderHandle *);
int pmu_counters_start(struct PMUCountersHandle *);
int pmu_counters_stop(struct PMUCountersHandle *);
int pmu_counters_peek_value(struct PMUCountersHandle *, int, uint64_t *);
int pmu_counters_peek_name(struct PMUCountersHandle *, int, size_t *, char *);

#if __cplusplus
}
#endif
#endif // LIB_PMU
