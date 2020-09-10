use crate::game::GameState;
use crate::visibility::{ CellVisibility, VisibilityAlgorithm };
use crate::world::{Layer, Tile, NpcType};
use chargrid::{
    app::{App as ChargridApp, ControlFlow},
    input::{keys, Input, KeyboardInput},
    render::{ColModify, Frame, View, ViewCell, ViewContext},
};
use coord_2d::Size;
use direction::CardinalDirection;
use rgb24::Rgb24;
use std::time::Duration;

struct AppData {
    game_state: GameState,
    visibility_algorithm: VisibilityAlgorithm,
}

impl AppData {
    fn new(
        screen_size: Size,
        rng_seed: u64,
        visibility_algorithm: VisibilityAlgorithm,
    ) -> Self {
        Self {
            game_state: GameState::new(screen_size, rng_seed, visibility_algorithm),
            visibility_algorithm,
        }
    }

    fn handle_input(&mut self, input: Input) {
        match input {
            Input::Keyboard(key) => match key {
                KeyboardInput::Left => self.game_state.maybe_move_player(CardinalDirection::West),
                KeyboardInput::Right => self.game_state.maybe_move_player(CardinalDirection::East),
                KeyboardInput::Up => self.game_state.maybe_move_player(CardinalDirection::North),
                KeyboardInput::Down => self.game_state.maybe_move_player(CardinalDirection::South),
                KeyboardInput::Char(' ') => self.game_state.wait_player(),
                _ => (),
            },
            _ => (),
        }
        self.game_state.update_visibility(self.visibility_algorithm);
    }
}

struct AppView {}

impl AppView {
    fn new() -> Self {
        Self {}
    }
}

fn currently_visible_view_cell_of_tile(tile: Tile) -> ViewCell {
    match tile {
        Tile::Floor => ViewCell::new()
            .with_character('.')
            .with_foreground(Rgb24::new_grey(63))
            .with_background(Rgb24::new(0, 0,63)),
        Tile::Npc(NpcType::Orc) => ViewCell::new()
            .with_character('o')
            .with_bold(true)
            .with_foreground(Rgb24::new(0, 187, 0)),
        Tile::Npc(NpcType::Troll) => ViewCell::new()
            .with_character('T')
            .with_bold(true)
            .with_foreground(Rgb24::new(187, 0, 0)),
        Tile::Player => ViewCell::new()
            .with_character('@')
            .with_foreground(Rgb24::new_grey(255)),
        Tile::Wall => ViewCell::new()
            .with_character('#')
            .with_foreground(Rgb24::new(0, 63, 63))
            .with_background(Rgb24::new(63, 127, 127)),
    }
}

fn previously_visible_view_cell_of_tile(tile: Tile) -> ViewCell {
    match tile {
        Tile::Floor => ViewCell::new()
            .with_character('.')
            .with_foreground(Rgb24::new_grey(63))
            .with_background(Rgb24::new_grey(0)),
        Tile::Npc(NpcType::Orc) => ViewCell::new()
            .with_character('o')
            .with_bold(true)
            .with_foreground(Rgb24::new_grey(63)),
        Tile::Npc(NpcType::Troll) => ViewCell::new()
            .with_character('T')
            .with_bold(true)
            .with_foreground(Rgb24::new_grey(63)),
        Tile::Player => ViewCell::new()
            .with_character('@')
            .with_foreground(Rgb24::new_grey(255)),
        Tile::Wall => ViewCell::new()
            .with_character('#')
            .with_foreground(Rgb24::new_grey(63))
            .with_background(Rgb24::new_grey(0)),
    }
}

impl<'a> View<&'a AppData> for AppView {
    // Frame represents the visible output of the app
    // calling set_cell_relative on it draws a character at that position

    // ColModify represents the color modifier
    // mainly used to dim the game area while a menu is visible

    // ViewContext allows a view to tell child views to render at an offset or 
    // with constraints. It's also a mechanism to pass color modifiers to child views

    // ViewCell is a character with a foreground and a background color, bold or underlined
    fn view<F: Frame, C: ColModify>(
        &mut self, 
        data: &'a AppData, 
        context: ViewContext<C>, 
        frame: &mut F
    ) {
        for entity_to_render in data.game_state.entities_to_render() {
            let view_cell = match entity_to_render.visibility {
                CellVisibility::Currently => {
                    currently_visible_view_cell_of_tile(entity_to_render.tile)
                }
                CellVisibility::Previously => {
                    previously_visible_view_cell_of_tile(entity_to_render.tile)
                }
                CellVisibility::Never => ViewCell::new(),
            };
            let depth = match entity_to_render.location.layer {
                None => -1,
                Some(Layer::Floor) => 0,
                Some(Layer::Feature) => 1,
                Some(Layer::Character) => 2,
            };
            frame.set_cell_relative(
                entity_to_render.location.coord,
                depth,
                view_cell,
                context
            );
        }
    }
}

pub struct App {
    data: AppData,
    view: AppView,
}

impl App {
    pub fn new(
        screen_size: Size,
        rng_seed: u64,
        visibility_algorithm: VisibilityAlgorithm,
    ) -> Self {
        Self {
            data: AppData::new(screen_size, rng_seed, visibility_algorithm),
            view: AppView::new(),
        }
    }
}

impl ChargridApp for App {
    fn on_input(&mut self, input: Input) -> Option<ControlFlow> {
        match input {
            Input::Keyboard(keys::ETX) | Input::Keyboard(keys::ESCAPE) => Some(ControlFlow::Exit),
            other => {
                self.data.handle_input(other);
                None
            }
        }
    }
    fn on_frame<F, C>(
        &mut self,
        _since_last_frame: Duration,
        view_context: ViewContext<C>,
        frame: &mut F,
    ) -> Option<ControlFlow>
    where
        F: Frame,
        C: ColModify,
    {
        self.view.view(&self.data, view_context, frame);
        None
    }
}