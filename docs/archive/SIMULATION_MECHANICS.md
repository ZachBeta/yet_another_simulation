# Simulation Mechanics

## Map Wrapping
- Toroidal world: x and y wrap via modulo arithmetic.
- Ensures continuous space; entities exiting one edge reappear on opposite.

## Agent Behavior
- **Orbital Motion**: agents orbit nearest enemy at fixed radius R.
  - Radial spring to maintain orbit distance (stiffness k).
  - Tangential velocity component for circular path.
- **Separation**: avoid overlapping with nearby agents (steer away within sepRange).

## Combat & Projectiles
- Agents fire **Bullets** when within `fireRange`.
  - Bullets have speed, TTL, and deal damage on hit.
  - Bullet collisions damage target and spawn a corpse on kill.

## Scavenging
- **Corpses** dropped on agent death persist as resources.
- Agents without living enemies seek nearest corpse.
  - Move toward corpse; on proximity (< pickupRange), heal and remove corpse.

## Update & Render Loop
1. Update bullets (move, wrap, detect collisions).
2. Update agents (combat, orbital motion, scavenging, wrap).
3. Clear canvas.
4. Draw corpses (gray dots), bullets (black dots), agents (team-colored circles).
5. Update stats UI.
