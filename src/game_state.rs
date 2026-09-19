mod bullet;
mod phys_object;
mod player;
mod setup_match;
mod wall;

use crate::player::Player;
use crate::scoring::{Leaderboard, PartialPoints};
use crate::shared::Hp;
use crate::{AppState, MenuState, scoring};
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
fn run_in_match<M>(
    systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
) -> ScheduleConfigs<ScheduleSystem> {
    systems
        .run_if(in_state(AppState::Match))
        .run_if(in_state(MenuState(false)))
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
        // When starting match, resume physics before setting up the match
        // Physics must be resumed first since it's needed to set up the match
        .add_systems(
            OnEnter(AppState::Match),
            (resume_physics, setup_match::setup_match).chain(),
        )
        // At the end of the match, pause the physics, clean up the match, and
        // update the leaderboard
        .add_systems(
            OnExit(AppState::Match),
            (
                pause_physics,
                setup_match::cleanup_match,
                scoring::update_leaderboard,
            ),
        )
        // Pause physics in the menu
        .add_systems(OnEnter(MenuState(true)), run_in_match(pause_physics))
        .add_systems(OnEnter(MenuState(false)), run_in_match(resume_physics))
        // At the beginning of each frame, clear the flag that indicates whether the fixed timestep has run this frame.
        .add_systems(PreUpdate, run_in_match(clear_fixed_timestep_flag))
        // At the beginning of each fixed timestep, set the flag that indicates whether the fixed timestep has run this frame.
        .add_systems(FixedPreUpdate, run_in_match(set_fixed_time_step_flag))
        // Advance the physics simulation using a fixed timestep.
        .add_systems(
            FixedUpdate,
            run_in_match((
                prepare_players.before(PhysicsSet::SyncBackend),
                update_players.after(PhysicsSet::Writeback),
            )),
        )
        .add_systems(
            FixedPostUpdate,
            run_in_match(
                (
                    (
                        (handle_player_hit, handle_player_damage, handle_wall_touch).chain(),
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
            run_in_match((
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
fn try_end_match(
    mut next_state: ResMut<NextState<AppState>>,
    leaderboard: Res<Leaderboard>,
    players: Query<(&Hp, &PartialPoints), With<Player>>,
) {
    let mut found_one = false;
    let mut full_point = false;
    for (hp, &partial_points) in players {
        full_point |= leaderboard.full_point(partial_points);
        if hp.hp > 0. {
            if found_one {
                return;
            } else {
                found_one = true;
            }
        }
    }

    // If at least one player has amassed enough partial points for a full
    // point, move to card selection Otherwise, start the next match.
    next_state.set(if full_point {
        AppState::CardSelection
    } else {
        AppState::Match
    });
}
