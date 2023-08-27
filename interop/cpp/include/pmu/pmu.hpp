#pragma once

#include "pmu/pmu.h"

#include <cassert>
#include <memory>
#include <optional>
#include <string>
#include <variant>

namespace pmu {
enum class CounterKind {
  Cycles = PMU_CYCLES,
  Instructions = PMU_INSTRUCTIONS,
  Branches = PMU_BRANCHES,
  BranchMisses = PMU_BRANCH_MISSES,
};

enum class CacheLevelKind {
  L1 = PMU_CACHE_L1,
  L1D = PMU_CACHE_L1D,
  L1I = PMU_CACHE_L1I,
  L2 = PMU_CACHE_L2,
  L3 = PMU_CACHE_L3,
  Last = PMU_CACHE_LAST,
  dTLB = PMU_CACHE_DTLB,
  iTLB = PMU_CACHE_ITLB,
  BPU = PMU_CACHE_BPU,
};

enum class CacheCounterKind {
  Hit = PMU_CACHE_HIT,
  Miss = PMU_CACHE_MISS,
};

enum class CacheOpKind {
  Read = PMU_CACHE_READ,
  Write = PMU_CACHE_WRITE,
  Prefetch = PMU_CACHE_PREFETCH,
};

struct CacheCounter {
  CacheLevelKind level;
  CacheCounterKind kind;
  CacheOpKind op;
};

using CounterKindAdvanced =
    std::variant<CounterKind, CacheCounter, std::string>;

class Builder;
class Counters;

struct CounterValue {
  std::string name;
  uint64_t value;
};

namespace detail {
class CountersIterator {
public:
  using value_type = CounterValue;

  CountersIterator(const Counters *counters, size_t id);
  CountersIterator() : mIsEnd(true), mId(-1), mCounters(nullptr) {}

  value_type operator*() { return *mValue; }

  CountersIterator &operator++() {
    CountersIterator it(mCounters, mId + 1);
    *this = it;
    return *this;
  }

  friend bool operator==(const CountersIterator &a, const CountersIterator &b) {
    if (a.mIsEnd == b.mIsEnd)
      return true;
    if (a.mId == b.mId)
      return true;

    return false;
  }

  friend bool operator!=(const CountersIterator &a, const CountersIterator &b) {
    if (a.mIsEnd == b.mIsEnd)
      return false;
    if (a.mId == b.mId)
      return false;

    return true;
  }

private:
  friend class Counters;

  bool mIsEnd;
  size_t mId;
  const Counters *mCounters;
  std::optional<CounterValue> mValue;
};
} // namespace detail

class Counters {
public:
  Counters(struct PMUCountersHandle *handle) : mHandle(handle) {}

  std::optional<CounterValue> peek(int id) const {
    std::string name;
    uint64_t value;
    size_t nameLen;

    if (pmu_counters_peek_name(mHandle, id, &nameLen, nullptr) != 0) {
      return std::nullopt;
    }

    name.resize(nameLen);
    if (pmu_counters_peek_name(mHandle, id, nullptr, name.data()) != 0) {
      return std::nullopt;
    }

    if (pmu_counters_peek_value(mHandle, id, &value) != 0) {
      return std::nullopt;
    }

    return CounterValue{.name = std::move(name), .value = value};
  }

  detail::CountersIterator begin() const {
    return detail::CountersIterator(this, 0);
  }

  detail::CountersIterator end() const { return detail::CountersIterator(); }

  void start() { pmu_counters_start(mHandle); }
  void stop() { pmu_counters_stop(mHandle); }

private:
  struct PMUCountersHandle *mHandle;
};

detail::CountersIterator::CountersIterator(const Counters *counters, size_t id)
    : mCounters(counters) {
  mValue = mCounters->peek(id);
  mIsEnd = !mValue;
  mId = mIsEnd ? -1 : id;
}

class Builder {
public:
  Builder() { mHandle = pmu_builder_create(); }

  Builder &add_counter(CounterKindAdvanced kind) {
    if (std::holds_alternative<CounterKind>(kind)) {
      pmu_builder_add_counter(
          mHandle, static_cast<PMUCounterKind>(std::get<CounterKind>(kind)),
          nullptr);
    } else if (std::holds_alternative<CacheCounter>(kind)) {
      auto cacheCounter = std::get<CacheCounter>(kind);
      pmu_builder_add_cache_counter(
          mHandle, static_cast<PMUCacheLevelKind>(cacheCounter.level),
          static_cast<PMUCacheCounterKind>(cacheCounter.kind),
          static_cast<PMUCacheOpKind>(cacheCounter.op));
    } else {
      pmu_builder_add_counter(mHandle, PMU_SYSTEM,
                              std::get<std::string>(kind).data());
    }

    return *this;
  }

  std::unique_ptr<Counters> build() {
    auto *handle = pmu_builder_build(mHandle);
    assert(handle);
    return std::make_unique<Counters>(handle);
  }

  ~Builder() { pmu_builder_release(mHandle); }

private:
  struct PMUBuilderHandle *mHandle;
};
} // namespace pmu
