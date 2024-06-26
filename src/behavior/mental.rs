use crate::{
    core::position::Ps, gameplay::gametime::{Time, TimeSpan}, utils::anyhashmap::{create_primitive_hashmap, PrimitiveHashMap, PrimitiveValue}
};

const MIN_PRIORITY: i32 = -1000;
pub const PRIORITY_ELEVATED: i32 = 5;
pub const PRIORITY_BASE: i32 = 1;
pub const PRIORITY_BASE_LOW: i32 = 0;
pub const PRIORITY_BASE_LOWES: i32 = -1;
pub const PRIORITY_HIGH: i32 = 10;
pub const PRIORITY_LOW_NEED: i32 = 3;
pub const PRIORITY_MEDIUM_NEED: i32 = PRIORITY_ELEVATED;
pub const PRIORITY_HIGH_NEED: i32 = PRIORITY_HIGH + 1;
pub const PRIORITY_DANGER: i32 = 100;
pub const PRIORITY_UNEVITABLE: i32 = 150;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum IntentionCompleted {
    Success,
    Failure,
    None,
    Undefined,
}

#[derive(Debug, Clone)]
pub struct MemoryEvent {
    pub name: String,
}

#[derive(Debug)]
pub struct Memory {
    pub events: Vec<MemoryEvent>,
    pub values: PrimitiveHashMap,
    _limit: usize,
    pub last_intention: Option<IntentionClass>,
    pub last_intention_failed: Option<IntentionClass>,
    pub last_intention_success: Option<IntentionClass>,
    pub last_intention_result: Option<bool>,
    pub cont_fail_counter: i32,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            events: Vec::new(),
            _limit: 0,
            values: create_primitive_hashmap(),
            last_intention: None,
            last_intention_failed: None,
            last_intention_success: None,
            last_intention_result: None,
            cont_fail_counter: 0
        }
    }

    pub fn insert_value(&mut self, key: &str, value: PrimitiveValue) {
        crate::utils::anyhashmap::insert_value(&mut self.values, key, value)
    }

    pub fn get_value(&self, key: &str) -> Option<PrimitiveValue> {
        crate::utils::anyhashmap::get_value(&self.values, key)
    }

    pub fn recall_all(&self) -> &Vec<MemoryEvent> {
        &self.events
    }

    pub fn recall_last(&self, length: usize) -> &[MemoryEvent] {
        &self.events[1..length]
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IntentionClass {
    MoveToDestination(Ps),
    MoveToPs(Ps),
    WaitCycles(isize),
    PickItemOfType(&'static str),
    ConsumeItemOfType(&'static str),
    ConsumeAnyItem(),
    PickAnyItem(),
    DropCarriedItemOfType(&'static str),
    DropAnyCarriedItem(),
    ConsumeCarriedItemOfType(&'static str),
    ConsumeAnyCarriedItem(),
    UseInteractiveOnce(),
    UseInteractiveCycles(isize),
    UseInteractiveMinutes(isize)
}

#[derive(Debug, Clone, Copy)]
pub struct Intention {
    pub priority: i32,
    pub value: IntentionClass,
}

impl Intention {
    pub fn new(priority: i32, value: IntentionClass) -> Intention {
        Intention { priority, value }
    }
}

#[derive(Debug)]
pub struct IntentionsCortex {
    queue: Vec<Intention>,
    current: Option<Intention>,
}

impl IntentionsCortex {
    pub fn new() -> IntentionsCortex {
        IntentionsCortex {
            queue: Vec::new(),
            current: None,
        }
    }

    pub fn intentions_left(&self) -> usize {
        let mut intentions_counter = self.queue.len();
        if self.current.is_some() {
            intentions_counter += 1
        }
        intentions_counter
    }

    pub fn clear_lower_than(&mut self, priority: i32) {
        while let Some(intention) = self.current {
            if intention.priority >= priority {
                return;
            }
            // clean this one up
            self.finish_current();
        }
    }

    pub fn clear_all(&mut self) {
        self.current = None;
        self.queue.clear();
    }

    fn enq(&mut self, i: Intention) {
        self.queue.push(i);
        self.queue.sort_by_key(|int| -int.priority)
    }

    fn dequeue(&mut self) -> Option<Intention> {
        if !self.queue.is_empty() {
            Some(self.queue.remove(0))
        } else {
            None
        }
    }

    fn max_priority_in_q(&self) -> Option<i32> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.queue.last().unwrap().priority)
        }
    }

    pub fn max_priority(&self) -> i32 {
        self.current
            .map(|i| i.priority)
            .unwrap_or(self.max_priority_in_q().unwrap_or(MIN_PRIORITY))
    }

    pub fn get_current(&self) -> &Option<Intention> {
        &self.current
    }

    pub fn finish_current(&mut self) -> bool {
        if self.current.is_none() {
            println!("WARN: Tried to finish current intention while no intention exists");
            false
        } else {
            // println!("Intention {:?} finished.", self.get_current());
            self.current = self.dequeue();
            // println!("Intention {:?} is next.", self.get_current());
            true
        }
    }

    pub fn intend(&mut self, i: Intention) {
        if let Some(current) = self.current {
            if current.priority < i.priority {
                // use new intention as current and enqueue previous current
                self.enq(current);
                self.current = Some(i);
            } else {
                // enqueue new intention
                self.enq(i)
            }
        } else {
            // we had no intentions previously - so this should be current
            self.current = Some(i)
        }
    }
}

#[derive(Debug)]
pub struct Brains {
    cycles_counter: isize,
    pub mem: Memory,
    pub intentions: IntentionsCortex,
    pub time_reference: Time
}

impl Brains {
    pub fn new() -> Self {
        Brains {
            mem: Memory::new(),
            intentions: IntentionsCortex::new(),
            cycles_counter: 0,
            time_reference: Time::new_zero()
        }
    }

    pub fn remember_intention_result(&mut self, intention: IntentionClass, success: bool) {
        self.mem.last_intention = Some(intention);
        if success {
            self.mem.last_intention_success = Some(intention);
            self.mem.cont_fail_counter = 0
        } else {
            self.mem.last_intention_failed = Some(intention);
            self.mem.cont_fail_counter += 1
        }
        self.mem.last_intention_result = Some(success);
    }

    pub fn init_cycles_counter(&mut self, cycles: isize) {
        self.cycles_counter = cycles
    }

    pub fn count_cycles(&mut self) -> bool {
        if self.cycles_counter <= 0 {
            return true;
        }
        self.cycles_counter -= 1;
        false
    }

    pub fn intend_cycles_count(&mut self, cycles: isize, priority: i32) {
        self.intentions
            .intend(Intention::new(priority, IntentionClass::WaitCycles(cycles)));
        self.init_cycles_counter(cycles)
    }

    pub fn save_time(&mut self, time: &Time) {
        self.time_reference = time.snapshot()
    }
}
