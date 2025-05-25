# Architecture

## Modules

- `index.html`: UI, control panel, canvas element.  
- `style.css`: Basic styling.  
- `script.js`: Core logic:
  - `Agent` class: behavior & drawing.  
  - `init()`: creates agents.  
  - `loop()`: updates, renders, and stats.  
  - Control handlers: start, pause, reset.

## Extension Points

- **Agent Behavior:**  
  - Replace or augment `update()` with ML-driven logic.  
- **Environment:**  
  - Add terrain/obstacle maps.  
- **UI Controls:**  
  - Add more inputs for stats or modes.
