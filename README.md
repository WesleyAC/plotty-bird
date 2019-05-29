# Plotty Bird

Plotty bird is an implementation of flappy bird which outputs to the HP7440A
pen plotter (or possibly other plotters which accept HP-GL input).

![Demo GIF](demo.gif)

To run it, simply plug in your HP7440A plotter, and
`cargo run -- /dev/ttyUSB0` (replacing `/dev/ttyUSB0`) with whatever the
plotter appears as on your system. Once the (randomized!) level has finished
drawing, press enter to begin the game, and press enter to jump. It works by
streaming HP-GL commands to the plotter in real time - the game gets around
20 "frames" per second.

If you would like more practical tools for working with the HP7440A (or other
HP plotters of a similar vintage), see [plotter-tools](https://github.com/WesleyAC/plotter-tools).
