# Yet Another Simulation

A WebAssembly-powered battle simulation running in the browser.

## Prerequisites

- Rust and Cargo (https://www.rust-lang.org/tools/install)
- wasm-pack (`cargo install wasm-pack`)
- Node.js v14+ and npm

## Building the WASM Module

```bash
cd sim_core
wasm-pack build --target web --out-dir ../wasm/pkg
```

## Installation

```bash
npm install
```

## Running Locally

```bash
npm start
```

Open [http://localhost:8000](http://localhost:8000) in your browser.

## Current State

- Four teams (orange, yellow, green, blue) battle each other in their quadrants.
- Agents use simple nearest-enemy targeting + separation and melee attack.
- Next: integrate neural-network decision making for approach/orbit/target/fire/salvage loop.

## Screenshots

![Screenshot 2025-05-04 12:35:25](./Screenshot%202025-05-04%20at%2012.35.25.png)

![Screenshot 2025-05-04 12:35:33](./Screenshot%202025-05-04%20at%2012.35.33.png)

![Screenshot 2025-05-04 12:35:56](./Screenshot%202025-05-04%20at%2012.35.56.png)

![Screenshot 2025-05-04 12:36:24](./Screenshot%202025-05-04%20at%2012.36.24.png)

![Screenshot 2025-05-04 12:36:47](./Screenshot%202025-05-04%20at%2012.36.47.png)
