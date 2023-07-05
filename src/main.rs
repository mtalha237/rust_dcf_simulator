//use delay::Delay;
//use std::time::Duration;

mod node;
mod scheduler;
mod theoretical;

use scheduler::{Scheduler};
use theoretical::calculate_tao_and_p;

const NUM_NODES:usize = 10;
const CW_MIN:usize = 32;
const MAX_MUL:u32 = 4;

fn main() {
    println!("... DCF simulator is started ...");
    
    let mut scheduler = Scheduler::new(NUM_NODES, CW_MIN, CW_MIN * 2_usize.pow(MAX_MUL));
    let mut more_events = true;

    calculate_tao_and_p(NUM_NODES, CW_MIN, MAX_MUL as i32);

    while more_events {
        more_events = scheduler.handle_next_event();
//        Delay::timeout(Duration::from_millis(1000));
    }
    scheduler.print_stats();
}