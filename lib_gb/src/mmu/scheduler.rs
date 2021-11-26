use std::collections::BinaryHeap;

#[derive(Clone, Copy, Debug)]
pub enum ScheduledEventType{
    Ppu,
    Timer
}

#[derive(Debug)]
pub struct ScheduledEvent{
    event_type:ScheduledEventType,
    cycles:u32
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.cycles.cmp(&other.cycles)
    }

    fn clamp(self, min: Self, max: Self) -> Self
    where Self: Sized {
        let cycles = self.cycles.clamp(min.cycles, max.cycles);
        Self{
            cycles,
            event_type:self.event_type
        }
    }

    fn max(self, other: Self) -> Self
    where Self: Sized {
        return if self.cycles >= other.cycles{
            self
        }
        else{
            other
        };
    }

    fn min(self, other: Self) -> Self
    where Self: Sized {
        return if self.cycles >= other.cycles{
            self
        }
        else{
            other
        };
    }
}

impl PartialOrd for ScheduledEvent{
    fn ge(&self, other: &Self) -> bool {
        self.cycles.ge(&other.cycles)
    }

    fn gt(&self, other: &Self) -> bool {
        self.cycles.gt(&other.cycles)
    }

    fn le(&self, other: &Self) -> bool {
        self.cycles.le(&other.cycles)
    }

    fn lt(&self, other: &Self) -> bool {
        self.cycles.lt(&other.cycles)
    }

    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.cycles.partial_cmp(&other.cycles)
    }
}

impl PartialEq for ScheduledEvent{
    fn eq(&self, other: &Self) -> bool {
        self.cycles.eq(&other.cycles)
    }

    fn ne(&self, other: &Self) -> bool {
        self.cycles.ne(&other.cycles)
    }
}

impl Eq for ScheduledEvent{}

pub struct Scheduler{
    events:Vec<ScheduledEvent>,
    m_cycles:u32
}

impl Default for Scheduler{
    fn default() -> Self {
        Self{
            events: Vec::new(),
            m_cycles:0
        }
    }
}

impl Scheduler{
    pub fn add_event(&mut self, mut event:ScheduledEvent){
        if self.events.is_empty(){
            self.events.push(event);
            return;
        }

        let mut index = 0;
        for i in (0..self.events.len()).rev(){
            if self.events[i].cycles < event.cycles{
                index = i;
                break;
            }
            // else{
            //     self.events[i].cycles -= event.cycles;
            // }
        }

        for i in index+1..self.events.len(){
            self.events[i].cycles -= event.cycles;
        }

        if index == 0{
            // event.cycles -= self.events[0].cycles;
            self.events.push(event);
            return;
        }

        // event.cycles -= self.events[index - 1].cycles;
        self.events.insert(index+1, event);
    }

    pub fn cycle(&mut self, m_cycles:u32)->Vec<ScheduledEvent>{
        let mut events = Vec::new();
        for _ in 0..m_cycles{

            self.m_cycles += 1;

            while m_cycles >= self.events[0].cycles{
                let mut event = self.events.remove(0);

                for e in &mut self.events{
                    e.cycles -= event.cycles;
                }

                event.cycles = self.m_cycles;
                events.push(event);
            }
        }

        return events;
    }
}

mod tests{
    use super::{ScheduledEvent, ScheduledEventType, Scheduler};

    #[test]
    pub fn add_event_test(){
        let mut scheduler = Scheduler::default();

        let event_type = ScheduledEventType::Ppu;
        scheduler.add_event(ScheduledEvent{event_type, cycles: 10});
        scheduler.add_event(ScheduledEvent{event_type, cycles: 100});
        scheduler.add_event(ScheduledEvent{event_type, cycles: 50});
        scheduler.add_event(ScheduledEvent{event_type, cycles: 1000});

        assert_eq!(scheduler.events, [ScheduledEvent{event_type, cycles: 10},
            ScheduledEvent{event_type, cycles: 40},
            ScheduledEvent{event_type, cycles: 50},
            ScheduledEvent{event_type, cycles: 900}]
        );
    }
}