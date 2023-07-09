use crate::node::{Node};
use std::fmt;

const NUM_STATISTICS:usize = 10000;

#[derive(Clone, Copy, PartialEq)]
pub enum EventType {
    DecrementBackoff,
    StartTx,
    EndTx
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::DecrementBackoff => write!(f, "Decrement Backoff"),
            EventType::StartTx => write!(f, "Start TX"),
            EventType::EndTx => write!(f, "End TX"),
        }
    }
}

pub struct Event {
    event_type: EventType,
    node_id: usize,
    time: u64
}

impl Event {
    pub fn new(event_type: EventType, node_id: usize, time: u64) -> Event {
        Event {
            event_type: event_type,
            node_id: node_id,
            time: time
        }
    }

    pub fn get_node_id(&self) -> usize {
        self.node_id
    }
}

pub struct Scheduler {
    event_list: Vec<Event>,
    nodes_in_tx: usize,
    use_rts_cts:bool,
    tx_success: bool,
    node_list: Vec<Node>,
    stop_stats: bool,
    time: u64
}

impl Scheduler {
    pub fn new(num_nodes: usize, use_rts_cts:bool, cw_min:usize, cw_max: usize) -> Scheduler {
        let mut node_list: Vec<Node> = Vec::new();
        let mut event_list: Vec<Event> = Vec::new();

        //Create all the nodes and kick-off them with backoff
        for i in 0..num_nodes {
            let node = Node::new(i, cw_min, cw_max);
            node_list.insert(i, node);
            let event = Event::new(EventType::DecrementBackoff, node.get_id(), 1);
            event_list.push(event);
        }

        Scheduler {
            event_list: event_list,
            nodes_in_tx: 0,
            use_rts_cts: use_rts_cts,
            tx_success: true,
            node_list: node_list,
            stop_stats: false,
            time: 0
        }
    }

    //Main function handling event loop
    pub fn handle_next_event (&mut self) -> bool {

        //Find the event with the lowest time and execute it
        let mut min_index = std::usize::MAX;
        let mut min_time: u64 = std::u64::MAX;
        for (pos, e) in self.event_list.iter().enumerate() {
            if e.time < min_time {
                min_index = pos;
                min_time = e.time;
            }
        }

        //This should not happen as the nodes will always TX
        if min_index == std::usize::MAX {
            println!("ERROR: NO MORE EVENTS IN THE LIST!");
            return false;
        }

        //Stop handling events if enough stats are collected
        if self.stop_stats {
            println!("Enough STATS collected!");
            return false;
        }

        let event = self.event_list.remove(min_index);
        let node_id: usize = event.get_node_id();
        
        self.time = event.time;
        match event.event_type {
            EventType::DecrementBackoff => {
                //See node.backoff() function for more details
                match self.node_list.get_mut(node_id).unwrap().backoff(event.time) {
                    Some(e ) => {
                        self.event_list.push(e);
                    },
                    None => return true,
                }
            },
            EventType::StartTx => {
                //Start the TX of a node and notify other nodes about channel occupation.
                //Note that if another has already decided to TX (but not started yet),
                //this will result in a collision. We keep track of the success of current TX
                //with the self.tx_success variable.
                match self.node_list.get_mut(node_id).unwrap().tx_start(event.time, self.use_rts_cts) {
                    Some(e ) => {
                        self.event_list.push(e);
                        if self.nodes_in_tx > 0 {
                            self.tx_success = false;
                        }
                        self.nodes_in_tx += 1;
                        for node in self.node_list.iter_mut() {
                            node.notify_channel(event.time, true, false, false);
                        }
                    },
                    None => return true,
                }
            },
            EventType::EndTx => {
                //Collect stats when a TX ends. Notify other channels if channel is free
                //so that they can restart backoff
                self.node_list.get_mut(node_id).unwrap().tx_end(self.tx_success);
                
                self.nodes_in_tx -= 1;
                if self.nodes_in_tx == 0 {
                    for node in self.node_list.iter_mut() {
                        match node.notify_channel(event.time, false, self.tx_success, self.use_rts_cts) {
                            Some(e) => {
                                self.event_list.push(e);
                            },
                            None => return true,
                        }
                    }
                    self.tx_success = true;
                }
                
                //See if enough stats are collected
                let (suc, fail) = self.node_list.get_mut(node_id).unwrap().get_stats();
                if suc + fail > NUM_STATISTICS {
                    self.stop_stats = true;
                }
            }
        };
        true
    }

    pub fn print_stats(&self) {
        let mut total_tx_bits:u64 = 0;
        println!("*******************Simulation results*******************");
        for node in self.node_list.iter() {
            let (suc, fail) = node.get_stats();
            println!("node {} prob success: {}", node.get_id(), (suc as f64) / ((suc + fail) as f64));
            total_tx_bits += node.get_tx_bits();
        }
        println!("Total throughput: {} Mbps", (total_tx_bits as f64) / (self.time as f64));
        println!("*******************Simulation results*******************");
    }
}