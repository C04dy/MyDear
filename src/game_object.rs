use crate::vector2::Vector2;
use colored::*;
use serde::{Deserialize, Serialize};

pub type GameObjectID = usize;

/// anything with this component should be moved with keyboard input
#[derive(Clone, Serialize, Deserialize)]
pub struct InputComponent;

/// this allows an object to move another object with this component
#[derive(Clone, Serialize, Deserialize)]
pub struct MoveableComponent;
/// this adds stats to an object to use it in combat
#[derive(Clone, Serialize, Deserialize)]
pub struct StatsComponent {
    pub strength: usize,
    pub agility: usize,
    pub defense: usize,
    pub luck: usize,
    pub max_health: usize,
    health: usize,
}
impl StatsComponent {
    pub fn new(
        strength: usize,
        agility: usize,
        defense: usize,
        luck: usize,
        max_health: usize,
    ) -> Self {
        StatsComponent {
            strength,
            agility,
            defense,
            luck,
            max_health,
            health: max_health,
        }
    }

    pub fn take_damage(&mut self, amount: usize) {
        self.set_health(self.health - amount);
    }
    pub fn heal(&mut self, amount: usize) {
        self.set_health(self.health + amount);
    }
    pub fn heal_to_max(&mut self) {
        self.set_health(self.max_health);
    }
    pub fn set_health(&mut self, amount: usize) {
        self.health = amount.clamp(0, self.max_health);
    }
    pub fn is_dead(&self) -> bool {
        return self.health == 0;
    }
    pub fn health(&self) -> usize {
        return self.health;
    }
    pub fn calculate_damage(&self, stat_to_attack: &StatsComponent) -> usize {
        let base = (self.strength as i32 - stat_to_attack.defense as i32).max(1) as usize;
        let bonus: usize = (0..self.luck)
            .filter(|_| rand::random::<f32>() < 0.20)
            .count();
        return base + bonus;
    }
}
#[derive(Clone)]
pub struct Dialogue {
    pub text: String,
    pub selections: Vec<String>,
    /// when a selection is selected the current_index variable in EventComponent will change to the number in the same index in this variable. if it is None the selection wont to anything. if the vector is empty none of them wont to anything
    pub selections_pointing_event: Vec<Option<usize>>,
    pub current_selection: usize,
}
impl Dialogue {
    pub fn new(
        text: String,
        selections: Vec<String>,
        selections_pointing_event: Vec<Option<usize>>,
        current_selection: usize,
    ) -> Self {
        Dialogue {
            text,
            selections,
            selections_pointing_event,
            current_selection,
        }
    }
}

pub const COMBAT_SELECTIONS: &[&str] = &["Fight", "Item", "Run"];
#[derive(Clone)]
pub enum TurnResult {
    WasPlayersTurn,
    WasEnemiesTurn,
    CombatEnded,
}
#[derive(Clone)]
pub enum CombatPhase {
    PlayerTurn,
    EnemyAttack(EnemyAttack),
    TurnResult(TurnResult),
}
#[derive(Debug, Clone, PartialEq)]
pub struct EnemyAttack {
    /// how many projectiles should enemy shoot
    pub projectile_count: usize,
    /// how many more projectiles still need to be spawned this round
    pub projectiles: Vec<Projectile>,
    /// internal timer for projectile movement
    pub move_timer: usize,
    /// internal timer for projectile spawning
    pub next_spawn_timer: usize,
    pub damage_dealt: usize,
}
impl EnemyAttack {
    pub fn new(combat: &Combat) -> Self {
        EnemyAttack {
            projectile_count: combat.projectile_count,
            projectiles: Vec::new(),
            move_timer: 0,
            next_spawn_timer: combat.projectile_spawn_time,
            damage_dealt: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Projectile {
    /// column; starts at the right edge of the screen, moves left each tick
    pub x: usize,
    /// lane row (0..2)
    pub row: usize,
    pub damage: usize,
}
#[derive(Clone)]
pub struct Combat {
    pub current_selection: usize,
    pub current_phase: CombatPhase,
    pub player_goes_first: bool,
    pub turn_order_decided: bool,
    /// ms turn result took
    pub turn_result_time: usize,
    /// internal timer for turn result
    pub turn_result_timer: usize,
    pub projectile_icon: ColoredString,
    pub projectile_damage: usize,
    pub projectile_count: usize,
    /// ms took to move a projectile
    pub projectile_move_time: usize,
    /// ms took to spawn the next projectile
    pub projectile_spawn_time: usize,
    /// row of the player when the enemy attacks
    pub player_row: usize,
    pub delete_when_defeated: bool, // maybe make this into a event so that we can destroy object after reading a dialogue?
}

impl Combat {
    pub fn new(
        current_phase: CombatPhase,
        player_goes_first: bool,
        turn_order_decided: bool,
        turn_result_time: usize,
        projectile_icon: ColoredString,
        projectile_damage: usize,
        projectile_count: usize,
        projectile_move_time: usize,
        projectile_spawn_time: usize,
        delete_when_defeated: bool,
    ) -> Self {
        Combat {
            current_selection: 0,
            current_phase,
            player_goes_first,
            turn_order_decided,
            turn_result_time,
            turn_result_timer: 0,
            projectile_icon,
            projectile_damage,
            projectile_count,
            projectile_move_time,
            projectile_spawn_time,
            player_row: 1,
            delete_when_defeated,
        }
    }
}

#[derive(Clone)]
pub enum GameEvent {
    None,
    Dialogue(Dialogue),
    Combat(Combat),
    TriggerObjectEvent(GameObjectID),
}

#[derive(Clone, Serialize, Deserialize)]
pub enum EventCondition {
    None,
}
impl EventCondition {
    pub fn next(&self) -> Self {
        match self {
            EventCondition::None => EventCondition::None,
        }
    }
    pub fn prev(&self) -> Self {
        match self {
            EventCondition::None => EventCondition::None,
        }
    }
}
pub fn event_condition_to_string(event_condition: &EventCondition) -> String {
    match event_condition {
        EventCondition::None => "None".to_string(),
    }
}

pub struct EventStep {
    pub event: GameEvent,
    pub requirement: EventCondition,
    pub repeat: bool,
    pub is_triggered: bool,
    /// if None it wont switch to any other event, if not will switch the event on that index (if it is a dialogue with selections that points to an event this value will not be used)
    pub next_event: Option<usize>,
}
impl EventStep {
    pub fn new(
        event: GameEvent,
        requirement: EventCondition,
        repeat: bool,
        next_event: Option<usize>,
    ) -> Self {
        EventStep {
            event,
            requirement,
            repeat,
            is_triggered: false,
            next_event,
        }
    }
}
pub struct EventComponent {
    pub events: Vec<EventStep>,
    pub current_index: usize,
}
impl EventComponent {
    pub fn new(events: Vec<EventStep>) -> Self {
        Self {
            events,
            current_index: 0,
        }
    }
}
pub struct GameObject {
    pub id: GameObjectID,
    pub icon: ColoredString,
    /// ALWAYS CHANGE POSITION FROM MAP FUNCTION, when the position changed, the hashmap on the map struct that point to the id should be changed, with that we can see if we should render the object
    pub position: Vector2,
}
