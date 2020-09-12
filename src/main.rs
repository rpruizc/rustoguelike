use app::App;
use chargrid_graphical::{Context, ContextDescriptor, Dimensions, FontBytes};
use coord_2d::Size;
use rand::Rng;
use simon::Arg;
use visibility::VisibilityAlgorithm;


mod app;
mod behaviour;
mod game;
mod terrain;
mod visibility;
mod ui;
mod world;

struct Args {
    rng_seed: u64,
    visibility_algorithm: VisibilityAlgorithm,
}

impl Args {
    fn parser() -> impl Arg<Item = Self> {
        simon::args_map! {
            let {
                rng_seed = simon::opt("r", "rng-seed", "seed for random number generator", "INT")
                    .with_default_lazy(|| rand::thread_rng().gen());
                visibility_algorithm = simon::flag("", "debug-omniscient", "enable omniscience")
                    .map(|omniscient| if omniscient {
                        VisibilityAlgorithm::Omniscient
                    } else {
                        VisibilityAlgorithm::Shadowcast
                    });
            } in {
                Self { rng_seed, visibility_algorithm }
            }
        }
    }
}

fn main() {
    const CELL_SIZE_PX: f64 = 24.;

    let Args {
        rng_seed,
        visibility_algorithm,
    } = Args::parser().with_help_default().parse_env_or_exit();
    println!("RNG Seed: {}", rng_seed);

    let context = Context::new(ContextDescriptor {
        font_bytes: FontBytes {
            normal: include_bytes!("./fonts/PxPlus_IBM_CGAthin.ttf").to_vec(),
            bold: include_bytes!("./fonts/PxPlus_IBM_CGA.ttf").to_vec(),
        },
        title: "RRRoguelike".to_string(),
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
    let app = App::new(screen_size, rng_seed, visibility_algorithm);
    context.run_app(app);
}