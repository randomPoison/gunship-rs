use std::cell::RefCell;

use ecs::*;
use scene::Scene;
use super::EntitySet;

pub type AlarmCallback = Fn(&Scene, Entity);

struct Alarm {
    start_time: f32,
    remaining_time: f32,
    callback: Box<Fn(&Scene, Entity)>,
    entity: Entity,
    id: AlarmId,
    repeating: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlarmId(usize);

pub struct AlarmManager {
    id_counter: usize,
    alarms: Vec<Alarm>,
    marked_for_destroy: RefCell<EntitySet>,
    pending_insertions: RefCell<Vec<Alarm>>,
    pending_cancellations: RefCell<Vec<AlarmId>>,
}

impl AlarmManager {
    pub fn new() -> AlarmManager {
        AlarmManager {
            id_counter: 0,
            alarms: Vec::new(),
            marked_for_destroy: RefCell::new(EntitySet::default()),
        }
    }

    pub fn assign<T>(&mut self, entity: Entity, duration: f32, callback: T) -> AlarmId
        where T: 'static + Fn(&Scene, Entity) {
        self.id_counter += 1;
        let id = AlarmId(self.id_counter);
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

    pub fn assign_repeating<T>(&mut self, entity: Entity, duration: f32, callback: T) -> AlarmId
        where T: 'static + Fn(&Scene, Entity) {
        self.id_counter += 1;
        let id = AlarmId(self.id_counter);
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

    pub fn cancel(&mut self, id: AlarmId) {
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
    fn destroy_all(&self, entity: Entity) {
        self.marked_for_destroy.borrow_mut().insert(entity);
    }

    fn destroy_marked(&mut self) {
        let mut marked_for_destroy = self.marked_for_destroy.borrow_mut();
        for entity in marked_for_destroy.drain() {
            // Iterate through all alarms and remove any associated with the entity.
            loop {
                let (index, remaining_time) = match self.alarms.iter().enumerate().find(|&(_, alarm)| alarm.entity == entity) {
                    Some((index, alarm)) => {
                        (index, alarm.remaining_time)
                    }
                    None => break,
                };

                if index < self.alarms.len() - 1 {
                    // There's another alarm after the one wer're removing, so update it.
                    self.alarms[index + 1].remaining_time += remaining_time;
                }

                self.alarms.remove(index);
            }
        }
    }
}

impl Clone for AlarmManager {
    fn clone(&self) -> AlarmManager {
        println!("WARNING: Cloning the alarm manager is not supported yet, hotloading will cancel all pending alarms");

        AlarmManager {
            id_counter: self.id_counter,
            alarms: Vec::new(),
            marked_for_destroy: self.marked_for_destroy.clone(),
        }
    }
}

pub struct AlarmSystem;

impl System for AlarmSystem {
    fn update(&mut self, scene: &Scene, delta: f32) {
        let mut callbacks_to_trigger: Vec<Alarm> = Vec::new();

        // The first time we borrow the alarm manager we collect up all the alarms that need
        // to be triggered, but we need to end our borrow of the alarm manager before triggering
        // them because the callback might try to borrow the alarm manager too.
        {
            let mut alarm_manager = scene.get_manager_mut::<AlarmManager>();

            let mut remaining_delta = delta;
            while alarm_manager.alarms.len() > 0 {
                {
                    let alarm = &mut alarm_manager.alarms[0];
                    if remaining_delta > alarm.remaining_time {
                        // Alarm is done, invoke the callback and then remove it from the manager.
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
                callbacks_to_trigger.push(alarm);
            }
        }

        // Do callbacks.
        for alarm in &callbacks_to_trigger {
            let entity = alarm.entity;
            (alarm.callback)(scene, entity);
        }

        // Once we've done all the callbacks then we can put the repeating alarms back into the
        // alarm manager.
        {
            let mut alarm_manager = scene.get_manager_mut::<AlarmManager>();
            for alarm in callbacks_to_trigger.drain(0..) {
                if alarm.repeating {
                    alarm_manager.insert_alarm(alarm);
                }
            }
        }
    }
}
