use crate::node::{Node};
use std::fmt;

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
    tx_success: bool,
    node_list: Vec<Node>,
    stop_stats: bool
}

impl Scheduler {
    pub fn new(num_nodes: usize, cw_min:usize, cw_max: usize) -> Scheduler {
        let mut node_list: Vec<Node> = Vec::new();
        let mut event_list: Vec<Event> = Vec::new();
        for i in 0..num_nodes {
            let node = Node::new(i, cw_min, cw_max);
            node_list.insert(i, node);
            let event = Event::new(EventType::DecrementBackoff, node.get_id(), 1);
            event_list.push(event);
        }

        Scheduler {
            event_list: event_list,
            nodes_in_tx: 0,
            tx_success: true,
            node_list: node_list,
            stop_stats: false
        }
    }

    pub fn add_event (&mut self, event: Event) {
        self.event_list.push(event);
    }

    pub fn handle_next_event (&mut self) -> bool {
        let mut min_index = std::usize::MAX;
        let mut min_time: u64 = std::u64::MAX;
        for (pos, e) in self.event_list.iter().enumerate() {
            if e.time < min_time {
                min_index = pos;
                min_time = e.time;
            }
        }

        if min_index == std::usize::MAX {
            println!("ERROR: NO MORE EVENTS IN THE LIST!");
            return false;
        }

        if self.stop_stats {
            println!("Enough STATS collected!");
            return false;
        }

        let event = self.event_list.remove(min_index);
        let node_id: usize = event.get_node_id();

        match event.event_type {
            EventType::DecrementBackoff => {
                match self.node_list.get_mut(node_id).unwrap().backoff(event.time) {
                    Some(e ) => {
                        self.add_event(e);
                    },
                    None => return true,
                }
            },
            EventType::StartTx => {
                match self.node_list.get_mut(node_id).unwrap().tx_start(event.time) {
                    Some(e ) => {
                        self.add_event(e);
                        if self.nodes_in_tx > 0 {
                            self.tx_success = false;
                        }
                        self.nodes_in_tx += 1;
                        for node in self.node_list.iter_mut() {
                            node.notify_channel(event.time, true, false);
                        }
                    },
                    None => return true,
                }
            },
            EventType::EndTx => {
                self.node_list.get_mut(node_id).unwrap().tx_end(self.tx_success);
                
                self.nodes_in_tx -= 1;
                if self.nodes_in_tx == 0 {
                    for node in self.node_list.iter_mut() {
                        match node.notify_channel(event.time, false, self.tx_success) {
                            Some(e) => {
                                self.event_list.push(e);
                            },
                            None => return true,
                        }
                    }
                    self.tx_success = true;
                }

                let (suc, fail) = self.node_list.get_mut(node_id).unwrap().get_stats();
//                println!("suc + fail: {}", suc + fail);
                if suc + fail > 10000 {
                    self.stop_stats = true;
                }
            }
        };

        return true;
    }

    pub fn print_stats(&self) {
        println!("*******************Simulation results*******************");
        for node in self.node_list.iter() {
            let (suc, fail) = node.get_stats();
            println!("node {} prob success: {}", node.get_id(), (suc as f64) / ((suc + fail) as f64));
        }
        println!("*******************Simulation results*******************");
    }
}