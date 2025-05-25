# Gameplay Loop

The simulation follows this main loop:

1. **Initialization**  
   - Create agents for each side based on configured counts and stats.  
   - Position agents randomly on the battlefield.

2. **Simulation Loop** (repeats every frame):  
   - **Update Agents:**  
     - Each agent finds the nearest living enemy.  
     - Move toward the target if out of attack range.  
     - Attack the target if within range (reduces health).  
   - **Render:**  
     - Clear the canvas.  
     - Draw each agent as a circle, opacity representing health.  
   - **Update Stats:**  
     - Count surviving agents per side.  
     - Display counts in the UI controls.

3. **Termination:**  
   - Loop continues until one side has zero surviving agents.  
   - Display final outcome: winning side and survivors.
