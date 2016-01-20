use callback::*;
use collections::EntitySet;
use ecs::*;
use engine::*;
use scene::Scene;
use std::cell::{Cell, RefCell};
use super::DefaultMessage;

#[derive(Debug, Clone)]
struct Alarm {
    start_time: f32,
    remaining_time: f32,
    callback: CallbackId,
    entity: Entity,
    id: AlarmId,
    repeating: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlarmId(usize);

impl Component for AlarmId {
    type Manager = AlarmManager;
    type Message = DefaultMessage<AlarmId>;
}

#[derive(Clone)]
pub struct AlarmManager {
    id_counter: Cell<usize>,
    alarms: RefCell<Vec<Alarm>>,
    callbacks: RefCell<CallbackManager<Fn(&Scene, Entity)>>,
    marked_for_destroy: RefCell<EntitySet>,
    pending_insertions: RefCell<Vec<Alarm>>,
    pending_cancellations: RefCell<Vec<AlarmId>>,
}

impl AlarmManager {
    pub fn new() -> AlarmManager {
        AlarmManager {
            id_counter: Cell::new(0),
            alarms: RefCell::new(Vec::new()),
            callbacks: RefCell::new(CallbackManager::new()),
            marked_for_destroy: RefCell::new(EntitySet::default()),
            pending_insertions: RefCell::new(Vec::new()),
            pending_cancellations: RefCell::new(Vec::new()),
        }
    }

    pub fn register_callback<F: 'static + Fn(&Scene, Entity)>(&self, callback: F) {
        self.callbacks.borrow_mut().register(CallbackId::of::<F>(), Box::new(callback));
    }

    #[allow(unused_variables)]
    pub fn assign<T>(&self, entity: Entity, duration: f32, callback: T) -> AlarmId
        where T: 'static + Fn(&Scene, Entity)
    {
        let callback_id = CallbackId::of::<T>();
        debug_assert!(
            self.callbacks.borrow().get(&callback_id).is_some(),
            "Cannot assign alarm callback which has not been registered");

        self.id_counter.set(self.id_counter.get() + 1);
        let id = AlarmId(self.id_counter.get());
        let alarm = Alarm {
            start_time: duration,
            remaining_time: duration,
            callback: callback_id,
            entity: entity,
            id: id,
            repeating: false,
        };

        self.pending_insertions.borrow_mut().push(alarm);

        id
    }

    #[allow(unused_variables)]
    pub fn assign_repeating<T>(&self, entity: Entity, duration: f32, callback: T) -> AlarmId
        where T: 'static + Fn(&Scene, Entity)
    {
        self.id_counter.set(self.id_counter.get() + 1);
        let id = AlarmId(self.id_counter.get());
        let alarm = Alarm {
            start_time: duration,
            remaining_time: duration,
            callback: CallbackId::of::<T>(),
            entity: entity,
            id: id,
            repeating: true,
        };

        self.pending_insertions.borrow_mut().push(alarm);

        id
    }

    pub fn cancel(&mut self, id: AlarmId) {
        // FIXME: This is wrong. When an alarm is remove the alarm after it needs to be given its
        // remaining time in order to keep the gap between them correct.
        self.alarms.borrow_mut().retain(|alarm| alarm.id != id);
    }

    fn process_pending_insertions(&self) {
        let mut pending_insertions = self.pending_insertions.borrow_mut();
        let mut alarms = self.alarms.borrow_mut();
        for mut new_alarm in pending_insertions.drain(..) {
            let mut insertion_index = None;
            new_alarm.remaining_time = new_alarm.start_time;
            for (index, alarm) in alarms.iter().enumerate() {
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
                alarms.insert(index, new_alarm);
            } else {
                // Alarm doesn't come before any other alarms, push it on the back.
                alarms.push(new_alarm);
            }
        }
    }
}

impl ComponentManagerBase for AlarmManager {}

impl ComponentManager for AlarmManager {
    type Component = AlarmId;

    fn register(builder: &mut EngineBuilder) {
        builder.register_manager(AlarmManager::new());
    }

    fn get(&self, _entity: Entity) -> Option<&Self::Component> {
        unimplemented!();
    }

    fn destroy(&self, entity: Entity) {
        self.marked_for_destroy.borrow_mut().insert(entity);
    }
}

derive_Singleton!(AlarmManager);

pub fn alarm_update(scene: &Scene, delta: f32) {
    let mut callbacks_to_trigger: Vec<Alarm> = Vec::new();

    // Process alarms marked for destruction and run any cleanup from last frame before
    // processing the alarms.
    {
        let alarm_manager = scene.get_manager::<AlarmManager>();
        let mut alarms = alarm_manager.alarms.borrow_mut();

        let mut marked_for_destroy = alarm_manager.marked_for_destroy.borrow_mut();
        for entity in marked_for_destroy.drain() {
            // Iterate through all alarms and remove any associated with the entity.
            loop {
                let (index, remaining_time) = {
                    let maybe_alarm =
                        alarms
                        .iter()
                        .enumerate()
                        .find(|&(_, alarm)| alarm.entity == entity);
                    match maybe_alarm {
                        Some((index, alarm)) => {
                            (index, alarm.remaining_time)
                        }
                        None => break,
                    }
                };

                if index < alarms.len() - 1 {
                    // There's another alarm after the one wer're removing, so update it.
                    alarms[index + 1].remaining_time += remaining_time;
                }

                alarms.remove(index);
            }
        }
    }

    // The first time we borrow the alarm manager we collect up all the alarms that need
    // to be triggered, but we need to end our borrow of the alarm manager before triggering
    // them because the callback might try to borrow the alarm manager too.
    {
        let alarm_manager = scene.get_manager::<AlarmManager>();

        alarm_manager.process_pending_insertions();

        let mut alarms = alarm_manager.alarms.borrow_mut();

        let mut remaining_delta = delta;
        while alarms.len() > 0 {
            {
                let alarm = &mut alarms[0];
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
            let alarm = alarms.remove(0);
            callbacks_to_trigger.push(alarm);
        }
    }

    {
        let alarm_manager = scene.get_manager::<AlarmManager>();
        let callbacks = alarm_manager.callbacks.borrow();

        // Do callbacks.
        for alarm in &callbacks_to_trigger {
            let entity = alarm.entity;
            let callback = callbacks.get(&alarm.callback).unwrap(); // TODO: Provide better panic message.
            callback(scene, entity);
        }
    }

    // Once we've done all the callbacks then we can put the repeating alarms back into the
    // alarm manager.
    {
        let alarm_manager = scene.get_manager::<AlarmManager>();
        let mut pending_insertions = alarm_manager.pending_insertions.borrow_mut();
        for alarm in callbacks_to_trigger.drain(0..) {
            if alarm.repeating {
                pending_insertions.push(alarm);
            }
        }
    }
}
