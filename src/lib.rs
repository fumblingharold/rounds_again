//! The solution is using a fixed timestep. This means that we advance the physics simulation by a fixed amount
//! at a time. If the real time that passed between two frames is less than the fixed timestep, we simply
//! don't advance the physics simulation at all.
//! If it is more, we advance the physics simulation multiple times until we catch up.
//! You can read more about how Bevy implements this in the documentation for
//! [`bevy::time::Fixed`](https://docs.rs/bevy/latest/bevy/time/struct.Fixed.html).
//!
//! This leaves us with a last problem, however. If our physics simulation may advance zero or multiple times
//! per frame, there may be frames in which the player's position did not need to be updated at all,
//! and some where it is updated by a large amount that resulted from running the physics simulation multiple times.
//! This is physically correct, but visually jarring. Imagine a player moving in a straight line, but depending on the frame rate,
//! they may sometimes advance by a large amount and sometimes not at all. Visually, we want the player to move smoothly.
//! This is why we need to separate the player's position in the physics simulation from the player's position in the visual representation.
//! The visual representation can then be interpolated smoothly based on the previous and current actual player position in the physics simulation.
//!
//! This is a tradeoff: every visual frame is now slightly lagging behind the actual physical frame,
//! but in return, the player's movement will appear smooth.
//! There are other ways to compute the visual representation of the player, such as extrapolation.
//! See the [documentation of the lightyear crate](https://cbournhonesque.github.io/lightyear/book/concepts/advanced_replication/visual_interpolation.html)
//! for a nice overview of the different methods and their respective tradeoffs.

mod game_state;
mod lobby_state;
mod pause_state;
mod player;
mod shared;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

/// Adds all the plugins and runs the app.
pub fn run_app() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_mod_debugdump::CommandLineArgs)
        .add_plugins(lobby_state::LobbyPlugin)
        .add_plugins(game_state::GamePlugin)
        .add_plugins(pause_state::PausePlugin)
        .init_state::<AppState>()
        .add_systems(Startup, (set_size_window, setup_camera))
        .run();
}

/// The different states the app can be in.
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    /// Before game, allows players to join or leave.
    #[default]
    Lobby,
    /// The actual game.
    Game,
    /// Game is paused.
    Pause,
}

/// Sets the window size to 1920x1080. This seems to have trouble on Mac, but needs to be rewritten so it scales
/// properly, so I'm not fixing it for now.
fn set_size_window(mut window: Single<&mut Window>) {
    window.resolution.set(1920., 1080.);
}

/// Prints the position of every entity.
fn print_entity_positions(positions: Query<(Entity, &Transform), With<RigidBody>>) {
    for (entity, transform) in positions.iter() {
        println!("Entity: {} position: {}", entity, transform.translation);
    }
}

/// Sets up the camera. Parameters are just left as default for now.
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
