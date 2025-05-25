# Inbox

## 2025-05-21
* scraper to fetch useful archived runs
* make sure training loop allows for salvage calls and training sessions include challenges that incentivize salvage

## 2025-05-19
* we should iterate toward training to fill the full complement of neural input parameters
* update the readme to explain how to run the whole training+tournament+battle pipeline

---
# WIP
* looks like toroidal got hallucinated back into default, let's fix that

* are ships not detecting distance across the wrap around boundaries? they all seem to converge into the middle, rather than loop around, there also might be some flickering of rendering when it's on the boundary, we might need to have some kind of buffer on the edges
    * this toroidal vs euclidian + walls issue somewhat works, but the ship agent AI seems to choke on both in different ways
    * also I keep wanting to be explicit that the module AI should be called ship agent or something similar to clearly communicate that it's a decision making module for a ship
    * most importantly the AI logic to determine which state it's in can fall thru and return nothing, and the lib step function is handling it silently - when it should potentially be considered an error? not sure
    * as it stands now 2025-05-06@1023 - most of our iteration is currently on the AI itself, so making moves toward a trainable AI is rising in priority, and we have a mostly functional NaiveAgent that we can use as a starting point for comparison training partner - have a neural network train against itself scoring total team health at the end of each game (which also means we need better detection of a team wipe and ending game when the agent team is wiped out). We can consider other success metrics after we have the basic in place. Once it's trained against itself enough, it can then play against a naive agent to see how it does.


# Ship Agent AI
* working on getting the current naive and vibe iterated code to stop getting stuck
* I wonder if world view can be condensed into a "scanner" that can give a 360 view, like a radar pulse so that the agent sight is a circle with a radius and the nearest point of interest on each sight line with an understanding of what is at that closest point, this would collapse the world view into a constant size and allow us to start using neural networks to make decisions


# UI
* when displaying diagnostics, display the cumulative calls for each command, as well as the current frame calls
* let's put a small border around the canvas, and make the background and other UI elements like buttons and stats more of a left rail layout


# Code Quality
* code quality - now that the project is starting to expand, we should address warnings and code cleanliness so it's all consistent - o4-mini-high was handling them in stride and ignoring them and making higher quality code changes, 04-mini-medium introduced some drift which is impacting 04-mini-high
    * perhaps add testing for each of the modules to make sure they all work correctly, there are a few with tests, but not all


# Soon
* shield cooldown, weapon cooldown, etc
    * refactor shield gen so it can have some more complex behavior, like regen after a delay, or regen at a rate that depends on the amount of damage taken without growing scope of the main loop


# Someday
* relativistic physics? map size? maybe we start with positions and velocities that are at a much smaller scale, like 1 meter and 1 meter per second - I can see this being applicable at scales of bacteria, up to galaxies like the powers of 10 videos from Carl Sagan. Eve Online has an interesting way of handling impulse drive vs warp drive vs jump drive to handle operating at different scales
* game state replay? at some point we could record the state of the game and allow the user to replay it