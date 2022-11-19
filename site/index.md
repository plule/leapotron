# Theremotion

[![Github](https://img.shields.io/badge/github-plule%2Ftheremotion-8da0cb?style=flat-square)](https://github.com/plule/leaprs)

👐 Theremotion is a virtual instrument inspired by the Theremin.

Just like a Theremin, the position of your hand in the air controls the pitch
and volume of the sound. However, you can also control much more dimensions of
the sound with specific motions.

With specific hand positions, you can control a low pass filter, a supersaw
sound, a guitar sound and more. You can even make the instrument "stick" to a
scale, making it much easier to control than a Theremin.

<iframe width="560" height="315" src="https://www.youtube.com/embed/GGALeKm_uzc" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>

## Features

- 👐 Theremotion is a synthesizer controlled by your hands.
- 👉 Move up and down your right hand to control the volume.
- 👈 Move up and down your left hand to control the pitch.
- 👋 Move your right hand on the horizontal plane to control the filter.
- 👋 Advance left hand on the horizontal plane to play a chord.
- 👌 Pinch with your left hand to stick on a scale.
- 🎸 Pinch with your right hand, and rotate it to play guitar.

## Installation

In order to play with Theremotion you need a [Leap Motion
Controller](https://www.ultraleap.com/product/leap-motion-controller/) and a
compatible Windows PC.

First of all, install the latest version of [Ultraleap's tracking
software](https://developer.leapmotion.com/tracking-software-download).
Theremotion was made for the Gemini software version. The previous version
(Orion) is also supported, though the hand tracking is not as good.

Next, download the latest Theremotion version from the [Github release
page](https://github.com/plule/theremotion/releases).

Extract the archive, and run the executable.

## Tools

[![Leap Motion Controller](doc/ultraleap.png)](https://www.ultraleap.com/product/leap-motion-controller/) [![Faust](doc/faust.png)]((https://faust.grame.fr/)) [![Rust](doc/rust.png)](https://www.rust-lang.org)

The hand tracking is provided by Ultraleap's [Leap Motion Controller](https://www.ultraleap.com/product/leap-motion-controller/).

The sound generation is created with Grame's [Faust](https://faust.grame.fr/).

The program is built with [Rust](https://www.rust-lang.org).