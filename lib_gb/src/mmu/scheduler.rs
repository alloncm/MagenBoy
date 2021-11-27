#[derive(Clone, Copy, Debug)]
pub enum ScheduledEventType{
    Ppu,
    Timer
}

#[derive(Debug)]
pub struct ScheduledEvent{
    pub event_type:ScheduledEventType,
    pub cycles:u32
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
    pub fn add_event(&mut self, event:ScheduledEvent){
        let mut index = self.events.len();
        for i in 0..self.events.len(){
            if self.events[i].cycles > event.cycles{
                index = i;
                break;
            }
        }

        self.events.insert(index, event);
    }

    pub fn cycle(&mut self, m_cycles:u32)->Vec<ScheduledEvent>{
        let mut events = Vec::new();
        for _ in 0..m_cycles{

            self.m_cycles += 1;

            while !self.events.is_empty() && self.m_cycles >= self.events[0].cycles{
                let event = self.events.remove(0);

                self.m_cycles -= event.cycles;
                for e in &mut self.events{
                    e.cycles -= event.cycles;
                }

                events.push(event);
            }
        }

        return events;
    }
}

mod tests{
    use super::{ScheduledEvent, ScheduledEventType, Scheduler};

    #[test]
    pub fn add_event_test1(){
        let mut scheduler = Scheduler::default();

        let event_type = ScheduledEventType::Ppu;
        scheduler.add_event(ScheduledEvent{event_type, cycles: 10});
        scheduler.add_event(ScheduledEvent{event_type, cycles: 100});
        scheduler.add_event(ScheduledEvent{event_type, cycles: 50});
        scheduler.add_event(ScheduledEvent{event_type, cycles: 1000});

        assert_eq!(scheduler.events, [ScheduledEvent{event_type, cycles: 10},
            ScheduledEvent{event_type, cycles: 50},
            ScheduledEvent{event_type, cycles: 100},
            ScheduledEvent{event_type, cycles: 1000}]
        );
    }

    #[test]
    pub fn add_event_test2(){
        let mut scheduler = Scheduler::default();

        let event_type = ScheduledEventType::Ppu;
        scheduler.add_event(ScheduledEvent{event_type, cycles: 2});
        scheduler.add_event(ScheduledEvent{event_type, cycles: 4});
        scheduler.add_event(ScheduledEvent{event_type, cycles: 1});
        scheduler.add_event(ScheduledEvent{event_type, cycles: 3});

        assert_eq!(scheduler.events, [ScheduledEvent{event_type, cycles: 1},
            ScheduledEvent{event_type, cycles: 2},
            ScheduledEvent{event_type, cycles: 3},
            ScheduledEvent{event_type, cycles: 4}]
        );

        let events = scheduler.cycle(1);

        assert_eq!(scheduler.events, [ScheduledEvent{event_type, cycles: 1},
            ScheduledEvent{event_type, cycles: 2},
            ScheduledEvent{event_type, cycles: 3}]
        );

        assert_eq!(events, [ScheduledEvent{event_type, cycles: 1}]);
    }

    #[test]
    pub fn add_event_test3(){
        let mut scheduler = Scheduler::default();

        let event_type = ScheduledEventType::Ppu;
        scheduler.add_event(ScheduledEvent{event_type, cycles: 4});
        scheduler.add_event(ScheduledEvent{event_type, cycles: 3});
        scheduler.add_event(ScheduledEvent{event_type, cycles: 2});
        scheduler.add_event(ScheduledEvent{event_type, cycles: 1});

        assert_eq!(scheduler.events, [ScheduledEvent{event_type, cycles: 1},
            ScheduledEvent{event_type, cycles: 2},
            ScheduledEvent{event_type, cycles: 3},
            ScheduledEvent{event_type, cycles: 4}]
        );
    }
}