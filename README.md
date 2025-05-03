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

## Screenshots

![Screenshot 1](./Screenshot%202025-04-27%20at%2009.48.23.png)
![Screenshot 2](./Screenshot%202025-04-27%20at%2009.48.26.png)
![Screenshot 3](./Screenshot%202025-04-27%20at%2009.48.28.png)
