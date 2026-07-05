mod bullet;
mod phys_object;
mod player;
mod shared;
mod wall;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bullet::*;
use phys_object::*;
use player::*;
use wall::setup_walls;

const PIXELS_PER_METER: f32 = 200.;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(PIXELS_PER_METER)
                .in_fixed_schedule(),
        )
        .add_plugins(RapierDebugRenderPlugin {
            default_collider_debug: ColliderDebug::AlwaysRender,
            enabled: true,
            style: DebugRenderStyle::default(),
            mode: DebugRenderMode::all(),
        })
        .init_resource::<DidFixedTimestepRunThisFrame>()
        .add_message::<BulletKillMessage>()
        .add_systems(Startup, (setup_walls, setup_phys_objects, setup_player))
        // At the beginning of each frame, clear the flag that indicates whether the fixed timestep has run this frame.
        .add_systems(PreUpdate, clear_fixed_timestep_flag)
        // At the beginning of each fixed timestep, set the flag that indicates whether the fixed timestep has run this frame.
        .add_systems(FixedPreUpdate, set_fixed_time_step_flag)
        // Advance the physics simulation using a fixed timestep.
        .add_systems(
            FixedUpdate,
            (
                prepare_players.before(PhysicsSet::SyncBackend),
                update_players.after(PhysicsSet::Writeback),
            ),
        )
        .add_systems(
            FixedPostUpdate,
            (
                (
                    handle_player_hit,
                    handle_phys_object_hit,
                    handle_bullet_collision,
                ),
                (kill_bullets, kill_players, kill_phys_objects),
            )
                .chain(),
        )
        .add_systems(
            // The `RunFixedMainLoop` schedule allows us to schedule systems to run before and after the fixed timestep loop.
            RunFixedMainLoop,
            (
                (
                    // Accumulate our input before the fixed timestep loop to tell the physics simulation what it should do during the fixed timestep.
                    update_input,
                )
                    .chain()
                    .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop),
                (
                    // Clear our accumulated input after it was processed during the fixed timestep.
                    // By clearing the input *after* the fixed timestep, we can still use `AccumulatedInput` inside `FixedUpdate` if we need it.
                    clear_input.run_if(did_fixed_timestep_run_this_frame),
                    // The player's visual representation needs to be updated after the physics simulation has been advanced.
                    // This could be run in `Update`, but if we run it here instead, the systems in `Update`
                    // will be working with the `Transform` that will actually be shown on screen.
                    //interpolate_rendered_transform,
                )
                    .chain()
                    .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop),
            ),
        );
    }
}
