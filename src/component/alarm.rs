use scene::Scene;
use ecs::*;

pub type AlarmCallback = Fn(&Scene, Entity);

struct Alarm {
    start_time: f32,
    remaining_time: f32,
    callback: Box<Fn(&Scene, Entity)>,
    entity: Entity,
    id: AlarmID,
    repeating: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlarmID(usize);

pub struct AlarmManager {
    id_counter: usize,
    alarms: Vec<Alarm>,
}

impl AlarmManager {
    pub fn new() -> AlarmManager {
        AlarmManager {
            id_counter: 0,
            alarms: Vec::new(),
        }
    }

    pub fn assign<T>(&mut self, entity: Entity, duration: f32, callback: T) -> AlarmID
        where T: 'static + Fn(&Scene, Entity) {
        self.id_counter += 1;
        let id = AlarmID(self.id_counter);
        let alarm = Alarm {
            start_time: duration,
            remaining_time: duration,
            callback: Box::new(callback),
            entity: entity,
            id: id,
            repeating: false,
        };

        self.insert_alarm(alarm);

        id
    }

    pub fn assign_repeating<T>(&mut self, entity: Entity, duration: f32, callback: T) -> AlarmID
        where T: 'static + Fn(&Scene, Entity) {
        self.id_counter += 1;
        let id = AlarmID(self.id_counter);
        let alarm = Alarm {
            start_time: duration,
            remaining_time: duration,
            callback: Box::new(callback),
            entity: entity,
            id: id,
            repeating: true,
        };

        self.insert_alarm(alarm);

        id
    }

    pub fn cancel(&mut self, id: AlarmID) {
        self.alarms.retain(|alarm| alarm.id != id);
    }

    fn insert_alarm(&mut self, mut new_alarm: Alarm) {
        let mut insertion_index = None;
        new_alarm.remaining_time = new_alarm.start_time;
        for (index, alarm) in self.alarms.iter().enumerate() {
            if new_alarm.remaining_time < alarm.remaining_time {
                // There's less time than the current alarm, which means we should insert
                // the new alarm before the current one.
                insertion_index = Some(index);
            } else {
                // There's more time remaining than the current alarm, so update the remaining
                // time by subtracting the current alarm's time.
                new_alarm.remaining_time -= alarm.remaining_time;
            }
        }

        if let Some(index) = insertion_index {
            self.alarms.insert(index, new_alarm);
        } else {
            // Alarm doesn't come before any other alarms, push it on the back.
            self.alarms.push(new_alarm);
        }
    }
}

impl ComponentManager for AlarmManager {
    fn destroy_all(&mut self, _entity: Entity) {
        unimplemented!();
    }
}

impl Clone for AlarmManager {
    fn clone(&self) -> AlarmManager {
        println!("WARNING: Cloning the alarm manager is not supported yet, hotloading will cancel all pending alarms");

        AlarmManager {
            id_counter: self.id_counter,
            alarms: Vec::new(),
        }
    }
}

pub struct AlarmSystem;

impl System for AlarmSystem {
    fn update(&mut self, scene: &Scene, delta: f32) {
        let mut alarm_manager = scene.get_manager_mut::<AlarmManager>();

        let mut remaining_delta = delta;
        while alarm_manager.alarms.len() > 0 {
            {
                let alarm = &mut alarm_manager.alarms[0];
                if remaining_delta > alarm.remaining_time {
                    // Alarm is done, invoke the callback and then remove it from the manager.
                    let entity = alarm.entity;
                    (alarm.callback)(scene, entity);
                    remaining_delta -= alarm.remaining_time;
                } else {
                    // Not enough delta time to finish the alarm,
                    // update its remaining time and break.
                    alarm.remaining_time -= remaining_delta;
                    break;
                }
            }

            // If the alarm shouldn't be removed the loop breaks before this point, so we're good
            // to remove the first alarm.
            let alarm = alarm_manager.alarms.remove(0);
            if alarm.repeating {
                alarm_manager.insert_alarm(alarm);
            }
        }
    }
}
