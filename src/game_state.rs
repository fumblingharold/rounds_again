mod bullet;
mod phys_object;
mod player;
mod setup_match;
mod wall;

use crate::AppState;
use crate::player::Player;
use crate::shared::Hp;
use bevy::ecs::schedule::ScheduleConfigs;
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bullet::*;
use phys_object::*;
use player::*;

const PIXELS_PER_METER: f32 = 200.;

pub struct GamePlugin;

/// Converts the given system into one that runs in state [`AppState::Game`].
fn run_in_game<M>(
    systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
) -> ScheduleConfigs<ScheduleSystem> {
    systems.run_if(in_state(AppState::Game))
}

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
        .add_systems(OnExit(AppState::Game), pause_physics)
        .add_systems(OnEnter(AppState::Game), resume_physics)
        .add_systems(
            OnTransition {
                exited: AppState::Lobby,
                entered: AppState::Game,
            },
            setup_match::setup_match,
        )
        .add_systems(
            OnTransition {
                exited: AppState::CardSelection,
                entered: AppState::Game,
            },
            setup_match::setup_match,
        )
        .add_systems(
            OnTransition {
                exited: AppState::Game,
                entered: AppState::CardSelection,
            },
            setup_match::cleanup_match,
        )
        // At the beginning of each frame, clear the flag that indicates whether the fixed timestep has run this frame.
        .add_systems(PreUpdate, run_in_game(clear_fixed_timestep_flag))
        // At the beginning of each fixed timestep, set the flag that indicates whether the fixed timestep has run this frame.
        .add_systems(FixedPreUpdate, run_in_game(set_fixed_time_step_flag))
        // Advance the physics simulation using a fixed timestep.
        .add_systems(
            FixedUpdate,
            run_in_game((
                prepare_players.before(PhysicsSet::SyncBackend),
                update_players.after(PhysicsSet::Writeback),
            )),
        )
        .add_systems(
            FixedPostUpdate,
            run_in_game(
                (
                    (
                        (handle_player_hit, handle_player_damage).chain(),
                        handle_phys_object_hit,
                        handle_bullet_hit,
                    ),
                    (kill_bullets, kill_players, kill_phys_objects),
                    try_end_match,
                )
                    .chain(),
            ),
        )
        .add_systems(
            // The `RunFixedMainLoop` schedule allows us to schedule systems to run before and after the fixed timestep loop.
            RunFixedMainLoop,
            run_in_game((
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
            )),
        );
    }
}

/// Pauses rapier game physics.
fn pause_physics(mut config: Single<&mut RapierConfiguration>) {
    config.physics_pipeline_active = false;
}

/// Resumes rapier game physics.
fn resume_physics(mut config: Single<&mut RapierConfiguration>) {
    config.physics_pipeline_active = true;
}

/// Ends the match if only one player is alive.
fn try_end_match(mut next_state: ResMut<NextState<AppState>>, players: Query<&Hp, With<Player>>) {
    let mut found_one = false;
    for hp in players {
        if hp.hp > 0. {
            if found_one {
                return;
            } else {
                found_one = true;
            }
        }
    }
    next_state.set(AppState::CardSelection);
}
