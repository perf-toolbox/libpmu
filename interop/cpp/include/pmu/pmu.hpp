#pragma once

#include "pmu/pmu.h"

#include <memory>
#include <optional>
#include <string>
#include <variant>

namespace pmu {
enum class CounterKind {
  Cycles = CYCLES,
  Instructions = INSTRUCTIONS,
  CacheMisses = CACHE_MISSES,
  Branches = BRANCHES,
  BranchMisses = BRANCH_MISSES,
};

using CounterKindAdvanced = std::variant<CounterKind, std::string>;

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
    } else {
      pmu_builder_add_counter(mHandle, SYSTEM,
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
