# Configuration

## Parameters

- `health`: starting health for each agent (default: 100)
- `speed`: movement speed per frame (default: 1.2)
- `attackRange`: distance within which agents can attack (default: 5)
- `attackDamage`: damage per attack per frame (default: 0.8)

### Extending Parameters

To expose new knobs (e.g., morale, formation):

1. Add input elements in `index.html` under `#controls`.
2. Read values in `script.js` in `init()` or Agent constructor.
3. Use new values in agent behavior (e.g., fleeing logic).
