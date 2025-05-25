# Agent Behavior & Future Evolution Roadmap

_Last updated: 2025-05-03_

This document outlines a phased plan for enhancing the AI agents in our simulation—from simple rule-based tactics through to trainable neural networks and evolutionary strategies.

## Phase 1: Health-Based Retreat
- **Trigger**: When self_health < X% (e.g. 30%).  
- **Behavior**: Agent computes nearest threat and applies thrust directly away.  
- **Outcome**: Dramatic breakaway maneuvers, ships “limp” to safety.
- **Next Step**: Add unit tests to assert agents flee rather than fire when health low.

## Phase 2: Orbit & Jitter Kiting
- **Trigger**: Enemy within range.  
- **Behavior**: Maintain distance at `attack_range - ε`, circle target using perpendicular thrust.  
- **Jitter**: Inject small random perpendicular Δv each tick.  
- **Outcome**: Smooth, weaving dogfights.  
- **Next Step**: Implement in `NaiveAgent.think()`, tune `ε` and jitter magnitude.

## Phase 3: Team Flank Rotation
- **Concept**: Assign roles on each team (e.g. “anchor” kiter, “wingmen” flankers).  
- **Behavior**: Wingmen calculate offset angle behind anchor→target line, rotate into position before firing.  
- **Outcome**: Coordinated, multi-ship tactics.  
- **Next Step**: Extend `WorldView` with simple role IDs; test small 3-ship formations.

## Phase 4: Attack/Kite/Flee State Machine
- **States**: Aggressive, Kiting, Retreat.  
- **Transitions**: Based on health, distance to enemy, and bullet density.  
- **Behavior**: Dynamically switch states for more life-like tactics.  
- **Outcome**: Agents shift strategies organically during battle.  
- **Next Step**: Add `agent.state` field, implement FSM in `think()`; unit-test transitions.

---

## Phase 5: Neural Network Agents
1. **Data Collection**: Log state→action pairs from rule-based agents.  
2. **Model Training**: Use a simple feed-forward network (e.g. 1 hidden layer) to predict `Action` given `WorldView`.  
3. **Inference**: Replace `NaiveAgent` with `NNAgent` for decision-making.  
4. **Evaluation**: Pit NN agents vs. rule-based, track win rates.  
- **Next Step**: Define state/action encoding and sketch training pipeline (e.g. Python + Rust FFI).

## Phase 6: Evolutionary Strategy & Team Diversity
- **Team Specialization**: Each team uses a different mix of rule-based and NN agents.  
- **Genetic Evolution**: Periodically evaluate top-performing NN weights, breed _crossover_ + _mutation_.  
- **Outcome**: Emergent meta-strategies; unpredictable, engaging battles.  
- **Next Step**: Research Rust crates for NEAT/ES, or integrate with Python GA libraries.

---

> This roadmap is a living document—update as new tactics emerge and we refine our simulation’s “vibes.”
