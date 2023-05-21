extern crate libpmu;

use libpmu::{Builder, CounterKind};

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
    let mut builder = Builder::new();
    builder.add_counter(CounterKind::Cycles);
    builder.add_counter(CounterKind::Instructions);
    builder.add_counter(CounterKind::Branches);
    builder.add_counter(CounterKind::BranchMisses);

    let mut counters = builder.build().unwrap();

    counters.start();
    let n = fib(50);
    counters.stop();

    println!("Fibonacci for 50 is {}", n);

    for c in counters.iter() {
        println!("{} is {}", c.kind.to_string(), c.value);
    }
}
