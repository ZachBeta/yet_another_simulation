* relativistic physics? map size? maybe we start with positions and velocities that are at a much smaller scale, like 1 meter and 1 meter per second - I can see this being applicable at scales of bacteria, up to galaxies like the powers of 10 videos from Carl Sagan. Eve Online has an interesting way of handling impulse drive vs warp drive vs jump drive to handle operating at different scales

* game state replay? at some point we could record the state of the game and allow the user to replay it

* shield cooldown, weapon cooldown, etc
* let's put a small border around the canvas, and make the background and other UI elements like buttons and stats more of a left rail layout
* are ships not detecting distance across the wrap around boundaries? they all seem to converge into the middle, rather than loop around, there also might be some flickering of rendering when it's on the boundary, we might need to have some kind of buffer on the edges
* perhaps add testing for each of the modules to make sure they all work correctly, there are a few with tests, but not all
* code quality - now that the project is starting to expand, we should address warnings and code cleanliness so it's all consistent - o4-mini-high was handling them in stride and ignoring them and making higher quality code changes, 04-mini-medium introduced some drift which is impacting 04-mini-high