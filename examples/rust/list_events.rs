extern crate libpmu;

fn main() {
    let events = libpmu::list_events();

    for e in &events {
        println!("{}", e.to_string())
    }
}
