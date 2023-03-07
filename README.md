# cloth_sim_rs
A cloth simulation based on verlet integration.
This is the newer, and more featureful version of the cloth sim.
**Very** basic audio was added.

This video series helped a lot:
https://youtu.be/3HjO_RGIjCU

## Screenshots




## Controls:

Hold LMB to do one of four things:
* Cut sticks
* Move points
* Place points
* Attract points

You can choose what the LMB should do by toggling the correct mode:
* Press [C] to choose cut-mode
* Press [H] to choose hand-mode (moving points)
* Press [P] to choose place-mode
* Press [F] to choose force-mode

A symbol in the top left corner of the screen will display what mode you're in.
The point closest to the mouse will be highlighted.
When you use hand-mode, that point will start following your cursor (along with all that are connected to it).

You can use the RMB to connect points together
by pressing down at one point and releasing at the other.

Additional keys:
* [Q] for debug information (if you feel so inclined)
* [A] to delete all sticks
* [W] to delete all points _and_ sticks
* [S] to make a point static
* [D] to reverse [S]
* And most importantly, **[G]** to generate a grid of connected points which forms a cloth

* There is also [T]. **Do not press it**. Ok, if you're feeling risky, i recommend trying it out when there are only around 5 dots on the screen.


### Compiling
* Clone the repository
* `cargo build --release`
* Optionally uncomment the options in Cargo.toml/profle.release to reduce the size of the binary

*LMB=Left Mouse Button,
*RMB=Right Mouse Button