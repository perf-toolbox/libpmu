extern crate pmu;

use pmu::{Builder, CounterKind};

fn fib(n: usize) -> usize {
    let mut a = 1;
    let mut b = 1;

    for _ in 1..n {
        let old = a;
        a = b;
        b += old;
    }
    b
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage: simple_stat <n>");
        return;
    }

    let n = args[1].parse::<usize>().unwrap();

    let mut builder = Builder::new();
    builder.add_counter(CounterKind::Cycles);
    builder.add_counter(CounterKind::Instructions);
    builder.add_counter(CounterKind::Branches);
    builder.add_counter(CounterKind::BranchMisses);
    builder.add_counter(CounterKind::CacheHits);
    builder.add_counter(CounterKind::CacheMisses);

    let events = pmu::list_events();

    events
        .first()
        .and_then(|e| Some(builder.add_counter(CounterKind::System(e.clone()))));

    let uops = events
        .iter()
        .find(|e| e.to_string() == "HW:RETIRED_UOPS" || e.to_string() == "HW:UOPS_RETIRED.SLOTS");
    uops.and_then(|e| Some(builder.add_counter(CounterKind::System(e.clone()))));

    let mut counters = builder.build().unwrap();

    counters.start();
    let f = fib(n);
    counters.stop();

    println!("Fibonacci for {} is {}", n, f);

    for c in counters.iter() {
        println!("{} is {}", c.kind.to_string(), c.value);
    }
}
