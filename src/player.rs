use std::any::Any;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Formatter};
use strum::FromRepr;
use crate::item::{Inventory, Item};

#[derive(Debug, Clone)]
pub struct DimensionId(String);

impl Default for DimensionId {
    fn default() -> Self {
        return Self::overworld();
    }
}

#[allow(dead_code)]
impl DimensionId {
    pub fn overworld() -> Self {
        return DimensionId("minecraft:overworld".to_string());
    }
    pub fn nether() -> Self {
        return DimensionId("minecraft:nether".to_string());
    }
    pub fn the_end() -> Self {
        return DimensionId("minecraft:the_end".to_string());
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, FromRepr)]
#[allow(dead_code)]
pub enum AttributeOperation {
    Add = 0,
    MultiplyBase = 1,
    Multiply = 2,
}

#[derive(Debug, Clone)]
pub struct AttributeModifier {
    pub name: String,
    pub amount: f64,
    pub operation: AttributeOperation,
    pub uuid: [i32; 4],
}

#[derive(Debug, Clone)]
pub struct EntityAttribute {
    pub base: f64,
    pub modifiers: Vec<AttributeModifier>,
}

#[derive(Debug, Clone)]
pub struct EntityFields {
    pub air: i16,
    pub custom_name: Option<String>,
    pub custom_name_visible: bool,
    pub fall_distance: f32,
    pub fire: i16,
    pub glowing: bool,
    pub has_visual_fire: bool,
    pub invulnerable: bool,
    pub motion: [f64; 3],
    pub no_gravity: bool,
    pub on_ground: bool,
    pub passenger: Vec<EntityBox>,
    pub portal_cool_down: i32,
    pub pos: [f64; 3],
    pub rotation: [f32; 2],
    pub silent: bool,
    pub tags: HashMap<String, ScoreboardTag>,
    pub ticks_frozen: i32,
    pub uuid: [i32; 4],
}

#[derive(Debug, Clone)]
pub struct ScoreboardTag {
    pub score: i32,
    pub locked: bool,
}

impl Default for EntityFields {
    fn default() -> Self {
        return Self {
            air: 0,
            custom_name: None,
            custom_name_visible: false,
            fall_distance: 0.0,
            fire: 0,
            glowing: false,
            has_visual_fire: false,
            invulnerable: false,
            motion: [0.0, 0.0, 0.0],
            no_gravity: false,
            on_ground: false,
            passenger: vec![],
            portal_cool_down: 0,
            pos: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0],
            silent: false,
            tags: Default::default(),
            ticks_frozen: 0,
            uuid: [0; 4],
        };
    }
}

#[derive(Debug, Clone)]
pub struct MobFields {
    pub absorption_amount: f32,
    pub active_effects: Vec<PotionEffect>,
    pub attributes: BTreeMap<String, EntityAttribute>,
    pub brain: HashMap<String, MobMemory>,
    pub death_time: i16,
    pub fall_flying: bool,
    pub hurt_by_time_stamp: i32,
    pub hurt_time: i16,
    pub sleeping_pos: [i32; 3],
}

impl Default for MobFields {
    fn default() -> Self {
        return Self {
            absorption_amount: 0.0,
            active_effects: vec![],
            attributes: Default::default(),
            brain: Default::default(),
            death_time: 0,
            fall_flying: false,
            hurt_by_time_stamp: 0,
            hurt_time: 0,
            sleeping_pos: [0, 0, 0],
        };
    }
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct MobMemory {
    ttl: i64,
    value: MemoryValue,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum MemoryValue {
    Boolean(bool),
    Integer(i32),
    Long(i64),
    UUID([i32; 4]),
    GlobalPos {
        dimension: DimensionId,
        pos: [i32; 3],
    },
    BlockPosList(Vec<[i32; 3]>),
    /// No tags, only the existence of memory is required
    Unit,
}

impl Default for MemoryValue {
    fn default() -> Self {
        return MemoryValue::Unit;
    }
}

#[derive(Debug, Clone)]
pub struct PotionEffect {
    /// Is added by beacon
    pub ambient: bool,
    /// level
    pub amplifier: i8,
    pub duration: i32,
    pub id: String,
    pub show_icon: bool,
    pub show_particles: bool,
    pub factor_calculation_data: PotionEffectFactorCalculationData,
    pub hidden_effects: Vec<PotionEffect>,
}

#[derive(Debug, Clone)]
pub struct PotionEffectFactorCalculationData {
    pub effect_changed_timestamp: i32,
    pub factor_current: f32,
    pub factor_previous_frame: f32,
    pub factor_start: f32,
    pub factor_target: f32,
    pub had_effect_last_tick: bool,
    pub padding_duration: i32,
}


#[derive(Debug, Clone)]
pub struct PlayerAbilities {
    pub flying: bool,
    pub instant_build: bool,
    pub invulnerable: bool,
    pub may_build: bool,
    pub may_fly: bool,
    pub walk_speed: f32,
    pub fly_speed: f32,
}

impl Default for PlayerAbilities {
    fn default() -> Self {
        return Self {
            flying: false,
            instant_build: false,
            invulnerable: false,
            may_build: true,
            may_fly: false,
            walk_speed: 1.0,
            fly_speed: 1.0,
        };
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, FromRepr)]
#[allow(dead_code)]
pub enum PlayerGameType {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}


#[derive(Debug, Clone)]
pub struct WardenSpawnTracker {
    pub cooldown_ticks: i32,
    pub ticks_since_last_warning: i32,
    pub warning_level: i32,
}

impl Default for WardenSpawnTracker {
    fn default() -> Self {
        return Self {
            cooldown_ticks: 0,
            ticks_since_last_warning: 0,
            warning_level: 0,
        };
    }
}

#[derive(Debug, Clone)]
pub struct XpInfo {
    pub level: i32,
    pub xp_p: f32,
    pub speed: i32,
    pub total: i32,
}

impl Default for XpInfo {
    fn default() -> Self {
        return Self {
            level: 0,
            xp_p: 0.0,
            speed: 0,
            total: 0,
        };
    }
}

#[derive(Debug, Clone)]
pub struct PlayerFields {
    pub abilities: PlayerAbilities,
    pub dimension: DimensionId,
    pub ender_items: Inventory,
    pub inventory: Inventory,
    pub entered_nether_position: Option<[f64; 3]>,
    pub food_exhaust_level: f32,
    pub food_saturation_level: f32,
    pub food_level: i32,
    pub food_tick_timer: i32,
    pub last_death_location: (DimensionId, [i32; 3]),
    pub game_type: PlayerGameType,
    pub previous_game_type: PlayerGameType,
    // recipe book
    pub root_vehicle: Option<[i32; 4]>,
    pub score: i32,
    pub seen_credits: bool,
    pub selected_item_slot: i32,
    pub sleep_timer: i16,

    pub should_entity_left: Option<EntityBox>,
    pub should_entity_right: Option<EntityBox>,
    pub spawn_angle: f32,
    pub spawn_dimension: DimensionId,
    pub spawn_forced: bool,
    pub spawn_pos: Option<[i32; 3]>,
    pub warden_spawn_tracker: WardenSpawnTracker,
    pub xp_info: XpInfo,
}

impl Default for PlayerFields {
    fn default() -> Self {
        return Self {
            abilities: Default::default(),
            dimension: DimensionId::default(),
            ender_items: Default::default(),
            inventory: Default::default(),
            entered_nether_position: None,
            food_exhaust_level: 0.0,
            food_saturation_level: 0.0,
            food_level: 0,
            food_tick_timer: 0,
            last_death_location: (Default::default(), [0, 0, 0]),
            game_type: PlayerGameType::Survival,
            previous_game_type: PlayerGameType::Survival,
            root_vehicle: None,
            score: 0,
            seen_credits: false,
            selected_item_slot: 0,
            sleep_timer: 0,
            should_entity_left: None,
            should_entity_right: None,
            spawn_angle: 0.0,
            spawn_dimension: Default::default(),
            spawn_forced: false,
            spawn_pos: None,
            warden_spawn_tracker: Default::default(),
            xp_info: Default::default(),
        };
    }
}

#[allow(dead_code)]
impl PlayerFields {
    fn selected_item(&self) -> Option<&Item> {
        let slot = self.selected_item_slot as i8;
        return self.inventory.0.get(&slot);
    }
    fn selected_item_mut(&mut self) -> Option<&mut Item> {
        let slot = self.selected_item_slot as i8;
        return self.inventory.0.get_mut(&slot);
    }
}

pub trait GetEntity {
    fn entity_fields(&self) -> &EntityFields;
    fn pos(&self) -> [f64; 3] {
        return self.entity_fields().pos;
    }
    fn to_mob_fields(&self) -> Option<&dyn GetMob> {
        return None;
    }
    fn is_living_body(&self) -> bool {
        return self.to_mob_fields().is_some();
    }

    fn clone_as_entity(&self) -> Box<dyn GetEntityMut>;
}

pub trait GetEntityMut: GetEntity {
    fn entity_fields_mut(&mut self) -> &mut EntityFields;
    fn to_mob_mut(&mut self) -> Option<&mut dyn GetMobMut> {
        return None;
    }
}

pub trait GetMob: GetEntity {
    fn mob_fields(&self) -> &MobFields;

    fn to_player(&self) -> Option<&dyn GetPlayer>;

    fn is_player(&self) -> bool {
        return self.to_player().is_some();
    }

    fn clone_as_living_body(&self) -> Box<dyn GetMobMut>;
}

pub trait GetMobMut: GetMob + GetEntityMut {
    fn mob_fields_mut(&mut self) -> &mut MobFields;
    fn to_player_mut(&mut self) -> Option<&mut dyn GetPlayerMut>;
}

pub trait GetPlayer: GetMob + GetEntity {
    fn player_fields(&self) -> &PlayerFields;

    fn clone_as_player(&self) -> Box<dyn GetPlayerMut>;
}

pub trait GetPlayerMut: GetPlayer + GetMobMut {
    fn player_fields_mut(&mut self) -> &mut PlayerFields;
}

#[derive(Debug, Clone, Default)]
pub struct Player {
    pub entity_fields: EntityFields,
    pub mob_fields: MobFields,
    pub player_fields: PlayerFields,
}

impl GetEntity for Player {
    fn entity_fields(&self) -> &EntityFields {
        return &self.entity_fields;
    }

    fn clone_as_entity(&self) -> Box<dyn GetEntityMut> {
        return Box::new(self.clone());
    }
}

impl GetEntityMut for Player {
    fn entity_fields_mut(&mut self) -> &mut EntityFields {
        return &mut self.entity_fields;
    }
}

impl GetMob for Player {
    fn mob_fields(&self) -> &MobFields {
        return &self.mob_fields;
    }

    fn to_player(&self) -> Option<&dyn GetPlayer> {
        return Some(self);
    }

    fn clone_as_living_body(&self) -> Box<dyn GetMobMut> {
        return Box::new(self.clone());
    }
}

impl GetMobMut for Player {
    fn mob_fields_mut(&mut self) -> &mut MobFields {
        return &mut self.mob_fields;
    }

    fn to_player_mut(&mut self) -> Option<&mut dyn GetPlayerMut> {
        return Some(self);
    }
}

impl GetPlayer for Player {
    fn player_fields(&self) -> &PlayerFields {
        return &self.player_fields;
    }

    fn clone_as_player(&self) -> Box<dyn GetPlayerMut> {
        return Box::new(self.clone());
    }
}

impl GetPlayerMut for Player {
    fn player_fields_mut(&mut self) -> &mut PlayerFields {
        return &mut self.player_fields;
    }
}

pub struct EntityBox(Box<dyn GetEntityMut>);

impl Debug for EntityBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ptr = self.0.as_ref() as *const dyn GetEntityMut;
        return write!(f, "EntityBox, type: {:?}, address: {:?}", self.0.type_id(), ptr);
    }
}

impl Clone for EntityBox {
    fn clone(&self) -> Self {
        return EntityBox(self.0.clone_as_entity());
    }
}