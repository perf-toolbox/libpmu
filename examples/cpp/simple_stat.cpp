#include <pmu/pmu.hpp>

#include <cstddef>
#include <cstdlib>
#include <iostream>

size_t fib(size_t n) {
  size_t a = 1;
  size_t b = 1;

  for (size_t i = 1; i < n; i++) {
    size_t old = a;
    a = b;
    b += old;
  }

  return b;
}

int main(int argc, char *argv[]) {
  int n = std::atoi(argv[1]);

  pmu::Builder builder;

  builder.add_counter(pmu::CounterKind::Cycles)
      .add_counter(pmu::CounterKind::Instructions)
      .add_counter(pmu::CounterKind::Branches)
      .add_counter(pmu::CacheCounter{pmu::CacheLevelKind::L1D,
                                     pmu::CacheCounterKind::Miss,
                                     pmu::CacheOpKind::Read})
      .add_counter(pmu::CounterKind::BranchMisses);

  auto counters = builder.build();

  counters->start();
  size_t f = fib(n);
  counters->stop();

  std::cout << "Fibonacci for " << n << " is " << f << "\n";

  for (auto value : *counters) {
    std::cout << value.name << ": " << value.value << "\n";
  }

  return 0;
}
