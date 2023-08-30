use bevy::prelude::*;
use seldom_state::prelude::*;

use crate::{cascade_input::button::ButtonJustPressedTrigger, util::state_machine::{insert_while_state, Timeout}};

use super::Magazine;


#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct Ready;
#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct Empty;
#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct Fire;
#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct Reload;


struct EmptyAmmoTrigger;
impl BoolTrigger for EmptyAmmoTrigger {
    type Param<'w, 's> = Query<'w, 's, &'static Magazine>;
    fn trigger(
        &self,
        entity: Entity,
        magazines: Self::Param<'_, '_>,
    ) -> bool {
        let Ok(magazine) = magazines.get(entity) else {
            warn!("Entity not found!");
            return false
        };
        magazine.is_empty()
    }
}
struct ReloadableTrigger;
impl BoolTrigger for ReloadableTrigger {
    // TODO optional &AmmoPool for out_of_ammo
    type Param<'w, 's> = Query<'w, 's, &'static Magazine>;
    fn trigger(
        &self,
        entity: Entity,
        magazines: Self::Param<'_, '_>,
    ) -> bool {
        let Ok(magazine) = magazines.get(entity) else {
            warn!("Entity not found!");
            return false
        };
        !magazine.is_full()  // TODO && !out_of_ammo
    }
}


pub struct SemiAutoStateMachine;
impl SemiAutoStateMachine {
    pub fn default_machine (fire_button: Entity, reload_button: Entity, fire_rate: f32, reload_time: f32) -> StateMachine {
        let machine = StateMachine::default()
            .trans::<Ready>(EmptyAmmoTrigger, Empty)
            .trans::<Ready>((ButtonJustPressedTrigger { button: fire_button }).and(EmptyAmmoTrigger.not()), Fire)
            .trans::<Ready>((ButtonJustPressedTrigger { button: reload_button }).and(ReloadableTrigger), Reload)
            .trans::<Empty>((ButtonJustPressedTrigger { button: reload_button }).and(ReloadableTrigger), Reload)
            // .trans::<Empty>(QueryFilterTrigger<With<AutoEmergencyReload>> + ReloadableTrigger, Reload)
            // .trans::<Empty>(fire_button + QueryFilterTrigger<With<FireToEmergencyReload>> + ReloadableTrigger, Reload)
            .trans::<Fire>(DoneTrigger::Success, Ready)
            .trans::<Reload>(DoneTrigger::Success, Ready)
            // .trans::<Reload>(QueryFilterTrigger<With<Canceled>>, Ready
            .set_trans_logging(true)
        ;
        let machine = insert_while_state::<Fire, _>(machine, Timeout::new(1.0/fire_rate));
        let machine = insert_while_state::<Reload, _>(machine, Timeout::new(reload_time));
        machine
    }
}
