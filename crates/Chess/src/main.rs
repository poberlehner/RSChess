pub mod controller;

use controller::Controller;
use std::time::*;

fn main() {
    let start_time = Instant::now();
    let mut app: Controller = Controller::new();
    println!("Starting time:  {:?}", start_time.elapsed());

    app.run();
}