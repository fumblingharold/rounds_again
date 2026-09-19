# Game Notes

## Nomenclature

- Match: A single fight between players the ends when only X of them are left

- Partial Point: The reward for doing well in a match, may be awarded based off
of survival or kills

- Full Point: The metric by which the game is scored and the winner is decided,
awarded for every X partial points a player has earned in the set

- Set: A series of matches that ends when at least one player has enough partial
points to form a full point, at which point any remaining partial points are
cleared (modulo!)

- Card Selection: After each set, players draw cards according to how many full
points they collected in the previous set (fewer full points -> more cards)

- Game: A series of sets followed by card selections

## Unimplemented Ideas

### Approved Ideas

- [ ] Add knockback when taking damage or firing bullet

- [ ] Add float time after legs stop touching ground

- [ ] Add block

- [ ] Make bullets a bit better in terms of not colliding with the firing player

- [ ] Kill phys-objects or bullets offscreen that won't return

- [ ] Force players back on screen and deal damage

- [ ] Apply damage from phys-object collisions

- [ ] Ammo clip with reload

- [ ] Character customization

- [X] Wall jump, implement with shape casting?

- [X] Prevent "legs" from colliding with bullets

### Ideas In Consideration

- [ ] Allow points awarded based on last survivor or num kills (num kills % n =
num points awarded, keep playing until threshold met)

- [ ] Players can choose between pre-made decks (classes) including an
all-rounder with all cards - Don't have a great idea for how to balance
custom decks (cards based on probability, not actually drawing cards)

- [ ] Allow players to join mid-game (num cards randomized or selected by other
players?)

- [ ] Countdown timer going into match

- [ ] Hold down to increase gravity

- [ ] Players hitboxes extend with points owned

- [ ] Players can't pause during their own card selection

- [ ] Setting to allow only Player1 to pause

- [ ]  Only show cards out of gameplay (menus)

- [ ]  Show points in some other cosmetic way

- [ ]  Use the colors and faces of the players next to their points

- [ ]  Colors of partial points are the same as the player killed to give them

- [ ]  An archive of cards and maps that allows the user to experiment and read
about any card

## Pending Fixes

- [ ] Make jump require button to be released before it can be reactivated (or
maybe touching ground also does it?)

- [ ] Players should start without jump stock
