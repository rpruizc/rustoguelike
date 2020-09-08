use chargrid_graphical::{Context, ContextDescriptor, Dimensions, FontBytes};
use chargrid::input::{keys, Input};
use coord_2d::{Coord, Size};
use rgb24::Rgb24;

struct App {
    data: AppData,
    view: AppView,
}

impl App {
    fn new(screen_size: Size) -> Self {
        Self{
            data: AppData::new(screen_size),
            view: AppView::new(),
        }
    }
}

impl chargrid::app::App for App {
    fn on_input(
        &mut self,
        input: chargrid::app::Input,
    ) -> Option<chargrid::app::ControlFlow> {
        match input {
            Input::Keyboard(keys::ETX) | Input::Keyboard(keys::ESCAPE) => {
                Some(chargrid::app::ControlFlow::Exit)
            }
            _ => None,
        }
    }

    fn on_frame<F, C>(
        &mut self,
        _since_last_frame: chargrid::app::Duration,
        view_context: chargrid::app::ViewContext<C>,
        frame: &mut F,
    ) -> Option<chargrid::app::ControlFlow>
    where
        F: chargrid::app::Frame,
        C: chargrid::app::ColModify,
    {
        use chargrid::render::View;
        self.view.view(&self.data, view_context, frame);
        None
    }
}

struct AppData {
    player_coord: Coord,
}

impl AppData {
    fn new(screen_size: Size) -> Self {
        Self {
            player_coord: screen_size.to_coord().unwrap() / 2,
        }
    }
}

struct AppView {}

impl AppView {
    fn new() -> Self {
        Self {}
    }
}

impl<'a> chargrid::render::View<&'a AppData> for AppView {
    // Frame represents the visible output of the app
    // calling set_cell_relative on it draws a character at that position

    // ColModify represents the color modifier
    // mainly used to dim the game area while a menu is visible

    // ViewContext allows a view to tell child views to render at an offset or 
    // with constraints. It's also a mechanism to pass color modifiers to child views

    // ViewCell is a character with a foreground and a background color, bold or underlined
    fn view<F: chargrid::app::Frame, C: chargrid::app::ColModify>(
        &mut self, 
        data: &'a AppData, 
        context: chargrid::render::ViewContext<C>, 
        frame: &mut F
    ) {
        let view_cell = chargrid::render::ViewCell::new()
            .with_character('R')
            .with_foreground(Rgb24::new_grey(255));
        frame.set_cell_relative(data.player_coord, 0, view_cell, context); 
    }
}

fn main() {
    const CELL_SIZE_PX: f64 = 24.;
    let context = Context::new(ContextDescriptor {
        font_bytes: FontBytes {
            normal: include_bytes!("./fonts/PxPlus_IBM_CGAthin.ttf").to_vec(),
            bold: include_bytes!("./fonts/PxPlus_IBM_CGA.ttf").to_vec(),
        },
        title: "Chargrid tut".to_string(),
        window_dimensions: Dimensions {
            width: 960.,
            height: 720.,
        },
        cell_dimensions: Dimensions {
            width: CELL_SIZE_PX,
            height: CELL_SIZE_PX,
        },
        font_dimensions: Dimensions {
            width: CELL_SIZE_PX,
            height: CELL_SIZE_PX,
        },
        font_source_dimensions: Dimensions {
            width: CELL_SIZE_PX as f32,
            height: CELL_SIZE_PX as f32,
        },
        underline_width: 0.1,
        underline_top_offset: 0.8,
    })
    .expect("Failed to initialize the graphical context");
    let screen_size = Size::new(40, 30);
    let app = App::new(screen_size);
    context.run_app(app);
}