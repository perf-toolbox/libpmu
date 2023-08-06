extern crate pmu;

fn main() {
    let events = pmu::list_events();

    for e in &events {
        println!("{}", e.to_string())
    }
}
