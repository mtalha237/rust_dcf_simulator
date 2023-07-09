mod node;
mod scheduler;
mod theoretical;

use scheduler::Scheduler;
use theoretical::calculate_tao_and_p;

const NUM_NODES:usize = 50;
const CW_MIN:usize = 128;
const MAX_MUL:u32 = 3;

fn main() {
    println!("... DCF simulator is started ...");
    
    let mut scheduler = Scheduler::new(NUM_NODES, CW_MIN, CW_MIN * 2_usize.pow(MAX_MUL));
    let mut more_events = true;

    //Print the results of theoretical calculations
    calculate_tao_and_p(NUM_NODES, CW_MIN, MAX_MUL as i32);

    //Do the simulation until enough stats are collected
    while more_events {
        more_events = scheduler.handle_next_event();
    }

    //Print the success probabilities resulted from simulation
    scheduler.print_stats();
}