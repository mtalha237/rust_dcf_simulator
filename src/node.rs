use std::fmt;
use crate::scheduler::{Event, EventType};

const SIFS:u64 = 28;
const DIFS:u64 = 128;
const SLOT_TIME:u64 = 50;
const PACKET_PAYLOAD:u64 = 8184;
const ACK_PAYLOAD:u64 = 112;
const MAC_HEADER:u64 = 272;
const PHY_HEADER:u64 = 128;
const PACKET_DURATION:u64 = PACKET_PAYLOAD + MAC_HEADER + PHY_HEADER;
const ACK_DURATION:u64 = ACK_PAYLOAD + PHY_HEADER;
const PROP_DELAY:u64 = 1;
const RTS_DURATION:u64 = 160 + PHY_HEADER;
const CTS_DURATION:u64 = 112 + PHY_HEADER;

const COMMON_DURATION_RTS_CTS:u64 = RTS_DURATION;
const SUCCESS_DURATION_RTS_CTS:u64 = 3 * SIFS + CTS_DURATION + PACKET_DURATION + ACK_DURATION + DIFS;
const COLLISION_DURATION_RTS_CTS:u64 = DIFS;

const COMMON_DURATION_BASIC:u64 = PACKET_DURATION;
const SUCCESS_DURATION_BASIC:u64 = SIFS + ACK_DURATION + DIFS;
const COLLISION_DURATION_BASIC:u64 = DIFS;


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
    cw_max: usize,
    tx_bits: u64,
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
            cw_max: cw_max,
            tx_bits: 0
        }
    }

    pub fn get_id (&self) -> usize {
        self.id
    }

    pub fn get_stats(&self) -> (usize, usize) {
        (self.num_success, self.num_fail)
    }

    pub fn get_tx_bits(&self) -> u64 {
        self.tx_bits
    }

    pub fn backoff (&mut self, time: u64) -> Option<Event> {
        match self.state {
            NodeStateType::Backoff => {
                //Decrement backoff in this state, if backoff reaches 0, start TX after 1us. This kind of simulates
                //propogation delay and allows for collisions etc.
                if self.backoff > 0 {
                    self.backoff -= 1;
                    return Some(Event::new (EventType::DecrementBackoff, self.id, time + SLOT_TIME));
                } else {
                    self.state = NodeStateType::InTx;
                    return Some(Event::new (EventType::StartTx, self.id, time + PROP_DELAY));
                }
            },
            NodeStateType::WaitChannel => {
                //Ignore backoff if channel is currently occupied.
                return None;
            },
            NodeStateType::InTx => {
                println!("ERROR: node {} was in tx when Backoff occured!", self.id);
                return None;
            },
        };
    }

    pub fn tx_start (&mut self, time: u64, use_rts_cts: bool) -> Option<Event> {
        //Creates a TX End event.
        if self.backoff != 0 {
            println!("ERROR: node {} backoff was not 0: {}", self.id, self.backoff);
            return None;
        }

        if self.state != NodeStateType::InTx {
            println!("ERROR: node {} was not In TX: {}", self.id, self.state);
            return None;
        }
        if !use_rts_cts {
            return Some(Event::new (EventType::EndTx, self.id, time + COMMON_DURATION_BASIC));
        } else {
            return Some(Event::new (EventType::EndTx, self.id, time + COMMON_DURATION_RTS_CTS));
        }
        
    }

    pub fn notify_channel (&mut self,
                            time: u64,
                            channel_occupied: bool, 
                            no_collision:bool,
                            use_rts_cts: bool) -> Option<Event> {
        //This function notifies nodes about channel occupation.

        if channel_occupied && self.state == NodeStateType::Backoff {
            //stop BACKOFF when the channel is occupied
            self.state = NodeStateType::WaitChannel;
            return None;
        } else if !channel_occupied && self.state == NodeStateType::WaitChannel {
            //start BACKOFF when the channel is free.
            //The time when next event occurs depends on success/fail and rts/cts 
            self.state = NodeStateType::Backoff;
            if !use_rts_cts {
                if no_collision {
                    return Some(Event::new (EventType::DecrementBackoff, self.id, time + SUCCESS_DURATION_BASIC));
                } else {
                    return Some(Event::new (EventType::DecrementBackoff, self.id, time + COLLISION_DURATION_BASIC));
                }
            } else {
                if no_collision {
                    return Some(Event::new (EventType::DecrementBackoff, self.id, time + SUCCESS_DURATION_RTS_CTS));
                } else {
                    return Some(Event::new (EventType::DecrementBackoff, self.id, time + COLLISION_DURATION_RTS_CTS));
                }
            }
            
        } else {
            return None;
        }
    }
    
    pub fn tx_end (&mut self, tx_success:bool) {
        //Collect statistics when TX ends.
        if tx_success {
            self.num_success += 1;
            self.tx_bits += PACKET_PAYLOAD;
            self.cw = self.cw_min;
        } else {
            self.num_fail += 1;
            self.cw *= 2;
            if self.cw > self.cw_max {
                self.cw = self.cw_max;
            }
        }

        self.backoff = rand::random::<usize>() % self.cw;
        self.state = NodeStateType::WaitChannel;
    }
}
