use crate::behaviour::Agent;
use crate::terrain::{self, TerrainTile};

use coord_2d::{Coord, Size};
use components::Components;
use direction::CardinalDirection;
use entity_table::{ComponentTable, Entity, EntityAllocator};
pub use layers::Layer;
use rand::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpcType {
    Orc,
    Troll,
}

impl NpcType {
    pub fn name(self) -> &'static str {
        match self {
            Self::Orc => "orc",
            Self::Troll => "troll",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Tile {
    Floor,
    Npc(NpcType),
    Player,
    Wall,
}

entity_table::declare_entity_module! {
    components {
        npc_type: NpcType,
        tile: Tile,
    }
}

spatial_table::declare_layers_module! {
    layers {
        character: Character,
        feature: Feature,
        floor: Floor,
    }
}

pub type Location = spatial_table::Location<Layer>;
type SpatialTable = spatial_table::SpatialTable<layers::Layers>;

pub struct Populate {
    pub player_entity: Entity,
    pub ai_state: ComponentTable<Agent>,
}

pub struct World {
    pub entity_allocator: EntityAllocator,
    pub components: Components,
    pub spatial_table: SpatialTable,
}

impl World {
    // NPCs can enter a cell even if it does contain a feature or another NPC
    pub fn can_npc_enter_ignoring_other_npcs(&self, coord: Coord) -> bool {
        self.spatial_table
            .layers_at(coord)
            .map(|layers| layers.feature.is_none())
            .unwrap_or(false)
    }

    // NPCs can enter a cell if it doesn't contain a feature or another NPC
    pub fn can_npc_enter(&self, coord: Coord) -> bool {
        self.spatial_table
            .layers_at(coord)
            .map(|layers| {
                let contains_npc = layers
                    .character
                    .map(|entity| self.components.npc_type.contains(entity))
                    .unwrap_or(false);
                let contains_feature = layers.feature.is_some();
                !(contains_npc || contains_feature)
            })
            .unwrap_or(false)
    }

    // To help with pathfinding, return the coordinate of an entity
    pub fn entity_coord(&self, entity: Entity) -> Option<Coord> {
        self.spatial_table.coord_of(entity)
    }

    pub fn maybe_move_character(&mut self, character_entity: Entity, direction: CardinalDirection) {
        let player_coord = self
            .spatial_table
            .coord_of(character_entity)
            .expect("player has no coord");
        let new_player_coord = player_coord + direction.coord();
        if new_player_coord.is_valid(self.spatial_table.grid_size()) {
            let dest_layers = self.spatial_table.layers_at_checked(new_player_coord);
            if let Some(character) = dest_layers.character {
                if let Some(npc_type) = self.components.npc_type.get(character) {
                    println!("Bump into the {}", npc_type.name());
                }
            } else if dest_layers.feature.is_none() {
                self.spatial_table
                    .update_coord(character_entity, new_player_coord)
                    .unwrap();
            }
        }
    }

    pub fn new(size: Size) -> Self {
        let entity_allocator = EntityAllocator::default();
        let components = Components::default();
        let spatial_table = SpatialTable::new(size);
        Self {
            entity_allocator,
            components,
            spatial_table,
        }
    }

    pub fn opacity_at(&self, coord: Coord) -> u8 {
        if self
            .spatial_table
            .layers_at_checked(coord)
            .feature
            .is_some()
        {
            255
        } else {
            0
        }
    }

    pub fn populate<R: Rng>(&mut self, rng: &mut R) -> Populate {
        let mut ai_state = ComponentTable::default();
        let mut player_entity = None;
        let terrain = terrain::generate_dungeon(self.spatial_table.grid_size(), rng);
        for (coord, &terrain_tile) in terrain.enumerate() {
            match terrain_tile {
                TerrainTile::Floor => self.spawn_floor(coord),
                TerrainTile::Npc(npc_type) => {
                    let entity = self.spawn_npc(coord,npc_type);
                    self.spawn_floor(coord);
                    ai_state.insert(entity, Agent::new());
                }
                TerrainTile::Player => {
                    self.spawn_floor(coord);
                    player_entity = Some(self.spawn_player(coord));
                }
                TerrainTile::Wall => {
                    self.spawn_floor(coord);
                    self.spawn_wall(coord);
                }
            }
        }
        Populate {
            ai_state,
            player_entity: player_entity.unwrap(),
        }
    }

    pub fn size(&self) -> Size {
        self.spatial_table.grid_size()
    }

    fn spawn_floor(&mut self, coord: Coord) {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Floor),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Floor);
    }

    fn spawn_npc(&mut self, coord: Coord, npc_type: NpcType) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Character),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Npc(npc_type));
        self.components.npc_type.insert(entity, npc_type);
        entity
    }

    fn spawn_player(&mut self, coord: Coord) -> Entity {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Character),
                },
            )
            .unwrap();
        self.components
            .tile
            .insert(entity, Tile::Player);
        entity
    }

    fn spawn_wall(&mut self, coord: Coord) {
        let entity = self.entity_allocator.alloc();
        self.spatial_table
            .update(
                entity,
                Location {
                    coord,
                    layer: Some(Layer::Feature),
                },
            )
            .unwrap();
        self.components.tile.insert(entity, Tile::Wall);
    }
}