use std::fmt;
use crate::scheduler::{Event, EventType};

const SIFS:u64 = 28;
const DIFS:u64 = 128;
const SLOT_TIME:u64 = 50;
const PACKET_DURATION:u64 = 8584;
const ACK_TIME:u64 = 72;

#[derive(Hash, Eq, Clone, Copy, PartialEq)]
pub enum NodeStateType {
    InTx,
    WaitChannel,
    Backoff
}

impl fmt::Display for NodeStateType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeStateType::InTx => write!(f, "In Tx"),
            NodeStateType::WaitChannel => write!(f, "Wait Channel"),
            NodeStateType::Backoff => write!(f, "Backoff"),
        }
    }
}

#[derive(Hash, Eq, Clone, Copy, PartialEq)]
pub struct Node {
    id: usize,
    backoff: usize,
    state: NodeStateType,
    num_success: usize,
    num_fail: usize,
    cw:usize,
    cw_min: usize,
    cw_max: usize
}

impl Node {
    pub fn new(id:usize, cw_min:usize, cw_max:usize) -> Node {
        let backoff: usize = rand::random::<usize>() % cw_min;
//        println!("node created with id: {} and backoff: {}", id, backoff);
        Node {
            id: id,
            backoff: backoff,
            state: NodeStateType::Backoff,
            num_success: 0,
            num_fail: 0,
            cw: cw_min,
            cw_min: cw_min,
            cw_max: cw_max
        }
    }

    pub fn get_id (&self) -> usize {
        self.id
    }

    pub fn get_stats(&self) -> (usize, usize) {
        (self.num_success, self.num_fail)
    }

    pub fn backoff (&mut self, time: u64) -> Option<Event> {
        match self.state {
            NodeStateType::Backoff => {
                if self.backoff > 0 {
                    self.backoff -= 1;
                    //println!("node {} backoff: {}", self.id, self.backoff);
                    return Some(Event::new (EventType::DecrementBackoff, self.id, time + SLOT_TIME));
                } else {
                    //println!("node {} will start TX!", self.id);
                    self.state = NodeStateType::InTx;
                    return Some(Event::new (EventType::StartTx, self.id, time + 1));
                }
            },
            NodeStateType::WaitChannel => {
                //Ignore
                return None;
            },
            NodeStateType::InTx => {
                println!("node {} was in tx when Backoff occured!", self.id);
                return None;
            },
        };
    }

    pub fn tx_start (&mut self, time: u64) -> Option<Event> {
        if self.backoff != 0 {
            println!("node {} backoff was not 0: {}", self.id, self.backoff);
            return None;
        }

        if self.state != NodeStateType::InTx {
            println!("node {} was not In TX: {}", self.id, self.state);
            return None;
        }

        return Some(Event::new (EventType::EndTx, self.id, time + PACKET_DURATION));
    }

    pub fn notify_channel (&mut self, time: u64, channel_occupied: bool, defer_by_ack:bool) -> Option<Event> {
        if channel_occupied && self.state == NodeStateType::Backoff {
            self.state = NodeStateType::WaitChannel;
            return None;
        } else if !channel_occupied && self.state == NodeStateType::WaitChannel {
            self.state = NodeStateType::Backoff;
            if defer_by_ack {
                return Some(Event::new (EventType::DecrementBackoff, self.id, time + DIFS + SIFS + ACK_TIME + 1));
            } else {
                return Some(Event::new (EventType::DecrementBackoff, self.id, time + DIFS + 1));
            }
        } else {
            return None;
        }
    }
    
    pub fn tx_end (&mut self, tx_success:bool) {
        if tx_success {
            self.num_success += 1;
            self.cw = self.cw_min;
        } else {
            self.num_fail += 1;
            self.cw *= 2;
            if self.cw > self.cw_max {
                self.cw = self.cw_max;
            }
        }
//        println!("node {} prob success: {}", self.id, (self.num_success as f64) / ((self.num_success + self.num_fail) as f64));
//        println!("node {} success: {}, fail: {}", self.id, self.num_success, self.num_fail);
        self.backoff = rand::random::<usize>() % self.cw;
        self.state = NodeStateType::WaitChannel;
    }
}
