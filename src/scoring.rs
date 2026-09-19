use bevy::prelude::*;

use crate::player::{Player, PlayerId};

/// Parts of a whole point. How many partial points make up a full point is
/// stored in the Leaderboard.
#[derive(Component, Debug, Default, Clone, Copy)]
pub struct PartialPoints(u8);

impl PartialPoints {
    /// Increments the number of partial points by 1.
    pub fn increment(&mut self) {
        self.0 += 1;
    }

    /// Resets the number of partial points to 0.
    pub fn reset(&mut self) {
        self.0 = 0;
    }
}

/// Adds points to each player in the leaderboard. Does NOT reset all players'
/// partial points.
pub fn update_leaderboard(
    query: Query<(&PlayerId, &PartialPoints), With<Player>>,
    mut leaderboard: ResMut<Leaderboard>,
) {
    let mut max_points = 0;
    let mut min_points = 255;

    // Update min and max and the leaderboard value
    for (player_id, partial_points) in query {
        let num_points = partial_points.0 / leaderboard.partial_points_per_point;
        max_points = u8::max(max_points, num_points);
        min_points = u8::min(min_points, num_points);
        leaderboard.full_points[player_id.0 as usize - 1] += num_points;
    }

    // Set max cards awarded to the max number of points
    leaderboard.max_cards_awarded = max_points;

    // If all players are to be awarded the same number of points, give them all
    // 1 card
    if max_points == min_points {
        leaderboard.max_cards_awarded = 1;
    };
}

/// The game's leaderboard.
#[derive(Resource, Debug)]
pub struct Leaderboard {
    full_points: Vec<u8>,
    partial_points_per_point: u8,
    max_cards_awarded: u8,
}

impl Leaderboard {
    pub fn new(num_players: u8) -> Self {
        Leaderboard {
            full_points: vec![0; num_players as usize],
            // TODO: allow setting of `partial_points_per_point` in some
            // settings menu
            partial_points_per_point: 2,
            max_cards_awarded: 0,
        }
    }

    /// Get the number of cards the player should take given the number of
    /// partial points.
    pub fn num_cards_to_take(&self, partial_points: PartialPoints) -> u8 {
        self.max_cards_awarded - partial_points.0 / self.partial_points_per_point
    }

    /// Whether the partial points are enough to form at least 1 full point.
    pub fn full_point(&self, partial_points: PartialPoints) -> bool {
        partial_points.0 >= self.partial_points_per_point
    }
}
