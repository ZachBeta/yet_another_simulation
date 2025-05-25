# 2v2 Browser Simulation Guide

This guide provides comprehensive instructions for setting up and using the 2v2 browser simulation, including champion selection, battle playback, and visualization.

## Table of Contents

1. [Prerequisites](#1-prerequisites)
2. [Architecture Overview](#2-architecture-overview)
3. [Setup Instructions](#3-setup-instructions)
4. [Running the Simulation](#4-running-the-simulation)
5. [Troubleshooting](#5-troubleshooting)
6. [Advanced Configuration](#6-advanced-configuration)
7. [Performance Optimization](#7-performance-optimization)
8. [Extending the Simulation](#8-extending-the-simulation)

## 1. Prerequisites

### System Requirements

- **Node.js** (v16 or later) and npm (v8 or later)
- **Rust** toolchain (stable)
- **wasm-pack** for compiling Rust to WebAssembly
- Modern web browser (Chrome, Firefox, Safari, or Edge)

### Project Structure

```
yet_another_simulation/
├── sim_core/           # Rust simulation core
│   └── out/            # Simulation outputs
│       ├── runs.json   # List of available simulation runs
│       └── <run_id>/   # Individual run directories
│           ├── champion_latest.json
│           └── elo_ratings.json
├── wasm/              # WebAssembly bindings
├── web/               # Frontend code
│   ├── index.html
│   ├── script.js
│   └── style.css
└── docs/              # Documentation
```

### runs.json Format

This file lists all available training runs and their key metrics:

```json
[
  {
    "run_id": "2v2-30s",
    "best_elo": 1234.5,
    "description": "2v2 with 30-second time limit"
  },
  {
    "run_id": "2v2-60s",
    "best_elo": 1278.2,
    "description": "2v2 with 60-second time limit"
  }
]
```

### HTML Markup

Ensure your `index.html` includes these elements:

```html
<label>
  Select Run:
  <select id="runSelect" style="width:200px;"></select>
</label>
<div id="runMeta" style="margin-left:8px; border:1px solid #ccc; padding:8px; max-width:300px;">
  <!-- Run metadata will be displayed here -->
</div>
```

## 2. Architecture Overview

### Data Flow

1. **Run Selection**
   - Load available runs from `sim_core/out/runs.json`
   - Display run metadata (ID, best Elo score)

2. **Champion Selection**
   - Load champions from `sim_core/out/<run_id>/elo_ratings.json`
   - Display champion list with Elo ratings

3. **Simulation Initialization**
   - Load champion metadata and genome
   - Initialize WebAssembly simulation
   - Set up rendering context

4. **Simulation Loop**
   - Update game state
   - Render frame
   - Handle user input
   - Manage playback controls

## 3. JavaScript Implementation

### 3.1 Global References

Add these at the top of your `script.js` after other element references:

```javascript
const runSelect = document.getElementById('runSelect');
const runMeta = document.getElementById('runMeta');
```

### 3.2 Loading Runs

Add this function to load and display available runs:

```javascript
async function loadRuns() {
  try {
    const response = await fetch('sim_core/out/runs.json');
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    const runs = await response.json();
    
    // Populate run selector
    runs.forEach(({ run_id, best_elo }) => {
      const label = `${run_id} (Best Elo: ${best_elo.toFixed(1)})`;
      runSelect.add(new Option(label, run_id));
    });
    
    // Handle run selection changes
    runSelect.onchange = () => {
      const selectedRun = runs.find(run => run.run_id === runSelect.value);
      if (selectedRun) {
        updateRunMetadata(selectedRun);
        loadEloRatings(selectedRun.run_id);
      }
    };
    
    // Initialize with first run
    if (runs.length > 0) {
      runSelect.selectedIndex = 0;
      runSelect.onchange();
    }
  } catch (error) {
    console.error('Failed to load runs:', error);
    runMeta.innerHTML = `Error loading runs: ${error.message}`;
  }
}

function updateRunMetadata(run) {
  runMeta.innerHTML = `
    <strong>Run:</strong> ${run.run_id}<br/>
    <strong>Best Elo:</strong> ${run.best_elo.toFixed(1)}<br/>
    ${run.description ? `<em>${run.description}</em>` : ''}
  `;
}
```

### 3.3 Update Initialization

Replace the default initialization with:

```javascript
// Start the application
async function init() {
  try {
    await loadRuns();
    requestAnimationFrame(loop);
  } catch (error) {
    console.error('Initialization failed:', error);
  }
}

// Start the application when the DOM is fully loaded
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
```

## 4. Setup Instructions

### 4.1 Install Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
cargo install wasm-pack

# Install Node.js dependencies
cd web
npm install
```

### 4.2 Configure runs.json

Create or update `sim_core/out/runs.json` with your run configurations:

```bash
mkdir -p sim_core/out
echo '[
  {
    "run_id": "2v2-30s",
    "best_elo": 1234.5,
    "description": "2v2 with 30-second time limit"
  },
  {
    "run_id": "2v2-60s",
    "best_elo": 1278.2,
    "description": "2v2 with 60-second time limit"
  }
]' > sim_core/out/runs.json
```

### 3.2 Configure Simulation Runs

Create or update `sim_core/out/runs.json`:

```json
[
  {
    "run_id": "2v2-30s",
    "best_elo": 1234.5,
    "description": "2v2 with 30-second time limit",
    "config": "configs/2v2_30s.toml"
  },
  {
    "run_id": "2v2-60s",
    "best_elo": 1278.2,
    "description": "2v2 with 60-second time limit",
    "config": "configs/2v2_60s.toml"
  }
]
```

### 3.3 Build the WebAssembly Module

```bash
# Build the WASM module
cd wasm
wasm-pack build --target web
```

## 4. Running the Simulation

### 4.1 Start the Development Server

```bash
# From the project root
cd web
npm start
```

This will start a local development server at `http://localhost:8000`.

### 4.2 Using the Web Interface

1. **Select a Run**
   - Choose from available runs in the dropdown
   - View run metadata (ID, best Elo score)

2. **Select Champions**
   - Choose champions from the champion dropdown
   - View champion statistics and metadata

3. **Control Playback**
   - **Start/Pause**: Toggle simulation playback
   - **Step**: Advance one simulation tick
   - **Reset**: Restart the simulation
   - **Speed**: Adjust simulation speed

4. **View Statistics**
   - Health bars for all agents
   - Current tick and simulation time
   - Performance metrics (FPS, tick duration)

## 5. Troubleshooting

### Common Issues

#### 5.1 "Failed to load runs.json"
- Ensure `sim_core/out/runs.json` exists and is valid JSON
- Check file permissions
- Verify the web server has access to the file

#### 5.2 WebAssembly Module Fails to Load
- Rebuild the WASM module: `cd wasm && wasm-pack build --target web`
- Clear browser cache or try a hard refresh (Ctrl+F5)
- Check browser console for specific error messages

#### 5.3 Performance Issues
- Reduce canvas resolution
- Decrease simulation complexity
- Enable WebGL rendering (if available)

## 6. Advanced Configuration

### 6.1 Customizing the UI

The web interface can be customized by modifying these files:

- `web/index.html`: HTML structure
- `web/style.css`: Styling
- `web/script.js`: Core functionality

### 6.2 Simulation Parameters

Simulation parameters can be adjusted in the configuration file specified in `runs.json`.

### 6.3 Adding Custom Visualizations

To add custom visualizations:

1. Create a new JavaScript module in `web/visualizations/`
2. Implement the required interface:
   ```javascript
   export default class CustomVisualization {
     constructor(canvas, context) {
       // Initialize visualization
     }
     
     update(simulationState) {
       // Update visualization based on simulation state
     }
     
     resize(width, height) {
       // Handle canvas resize
     }
   }
   ```
3. Import and register the visualization in `script.js`

## 7. Performance Optimization

### 7.1 WebAssembly Optimizations

- Build with optimizations: `wasm-pack build --release --target web`
- Enable WebAssembly SIMD support (if available)
- Use `wee_alloc` for smaller code size

### 7.2 Rendering Optimizations

- Use `requestAnimationFrame` for smooth animations
- Implement viewport culling
- Use sprite sheets for efficient rendering

### 7.3 Memory Management

- Reuse objects to reduce garbage collection
- Use TypedArrays for large datasets
- Monitor memory usage with browser dev tools

## 8. Extending the Simulation

### 8.1 Adding New Champion Types

1. Add champion definition to `sim_core/src/champions/`
2. Update the champion factory
3. Add corresponding visualization

### 8.2 Implementing New Game Modes

1. Define game mode rules in `sim_core/src/game_modes/`
2. Update the simulation state machine
3. Add UI controls for the new mode

### 8.3 Integrating with External Services

- **Analytics**: Add tracking for user interactions
- **Multiplayer**: Implement WebSocket communication
- **Persistence**: Save and load simulation states

## 9. Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## 10. License

[Your License Here]

## 11. Support

For support, please open an issue on GitHub or contact the maintainers.
