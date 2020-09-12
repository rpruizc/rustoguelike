use crate::behaviour::{Agent, BehaviourContext, NpcAction};
use crate::visibility::{CellVisibility, VisibilityAlgorithm, VisibilityGrid};
use crate::world::{Location, Populate, Tile, World};

use coord_2d::Size;
use direction::CardinalDirection;
use entity_table::{ComponentTable, Entity};
use rand::SeedableRng;
use rand_isaac::Isaac64Rng;

// A type is defined to tell the renderer what needs to be rendered. In this case
// a given tile a t a given position on screen
pub struct EntityToRender {
    pub tile: Tile,
    pub location: Location,
    pub visibility: CellVisibility,
}

pub struct GameState {
    ai_state: ComponentTable<Agent>,
    behaviour_context: BehaviourContext,
    player_entity: Entity,
    shadowcast_context: shadowcast::Context<u8>,
    visibility_grid: VisibilityGrid,
    world: World,
}

impl GameState {
    fn ai_turn(&mut self) {
        self.behaviour_context
            .update(self.player_entity, &self.world);
        let dead_entities = self  // before all the NPCs take their turn, remove dead NPCs
            .ai_state
            .entities()
            .filter(|&entity| !self.world.is_living_character(entity))
            .collect::<Vec<_>>();
        for dead_entity in dead_entities {
            self.ai_state.remove(dead_entity);
        }
        for (entity, agent) in self.ai_state.iter_mut() {
            let npc_action = agent.act(
                entity,
                self.player_entity,
                &self.world,
                &mut self.behaviour_context,
            );
            match npc_action {
                NpcAction::Wait => (),
                NpcAction::Move(direction) => self.world.maybe_move_character(entity, direction),
            }
        }
    }

    pub fn is_player_alive(&self) -> bool {
        self.world.is_living_character(self.player_entity)
    }

    pub fn new(
        screen_size: Size,
        rng_seed: u64,
        initial_visibility_algorithm: VisibilityAlgorithm,
    ) -> Self {
        let mut world = World::new(screen_size);
        let mut rng = Isaac64Rng::seed_from_u64(rng_seed);
        let Populate {
            ai_state,
            player_entity,
        } = world.populate(&mut rng);
        let behaviour_context = BehaviourContext::new(screen_size);
        let shadowcast_context = shadowcast::Context::default();
        let visibility_grid = VisibilityGrid::new(screen_size);
        let mut game_state = Self {
            ai_state,
            behaviour_context,
            player_entity,
            shadowcast_context,
            visibility_grid,
            world,
        };
        game_state.update_visibility(initial_visibility_algorithm);
        game_state
    }

    pub fn maybe_move_player(&mut self, direction: CardinalDirection) {
        self.world
            .maybe_move_character(self.player_entity, direction);
        self.ai_turn();
    }

    // Method returns an iterator over EntityToRender for all the entities
    pub fn entities_to_render<'a>(&'a self) -> impl 'a + Iterator<Item = EntityToRender> {
        let tile_component = &self.world.components.tile;
        let spatial_table = &self.world.spatial_table;
        let visibility_grid = &self.visibility_grid;
        tile_component.iter().filter_map(move |(entity, &tile)| {
            let &location = spatial_table.location_of(entity)?;
            let visibility = visibility_grid.cell_visibility(location.coord);
            Some(EntityToRender {
                tile,
                location,
                visibility,
            })
        })
    }

    pub fn update_visibility(&mut self, visibility_algorithm: VisibilityAlgorithm) {
        let player_coord = self
            .world
            .spatial_table
            .coord_of(self.player_entity)
            .unwrap();
        self.visibility_grid.update(
            player_coord,
            &self.world,
            &mut self.shadowcast_context,
            visibility_algorithm,
        );
    }

    pub fn wait_player(&mut self) {
        self.ai_turn();
    }
}
