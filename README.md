# N-body simulation in Rust

![Static Badge](https://img.shields.io/badge/Bevy-0.16.0-green)
![Static Badge](https://img.shields.io/badge/Bevy_egui-0.34.1-green)
![Static Badge](https://img.shields.io/badge/Rustc-1.85.0-green)


![Simulation gif](https://github.com/NatasjaVitoft/N-body-simulation-in-Rust/blob/main/images/n-body.gif)


This project is a simple n-body simulation written in Rust, using the Bevy game engine for update loop and rendering engine. It is created as an exam project for our Rust course in spring 2025.

Bodies are generated with random mass and position. Maximum and minimum mass are defined within parameters which are tweakable at runtime.
At the moment the only rules for body movement is mutual attraction.

Body attraction is calculated using Barnes-Hut algorithm.

Mass of the bodie is represented by their size and and color in relation to each other. This means that heavier generated objects are bigger and more red in colour, than more light bodies, which are smaller and more green.

A GUI is available for for tweaking different parameters in the simulation. Some are live-tweakable while the simluation is running, others need a restart.

Parameters available are:

**Live Tweakables**:
- **G** (Gravity constant)
- **Delta T** (time-step approximation)
- **Show Quadtree** (Draws the quadtree structure used for Barnes-hut algo)

**Needs Restart**:
- **Min Body Mass** (Minimum mass possibly generated)
- **Max Body Mass** (Maximum mass possibly generated. Overwrites min when lower than min)
- **Num Bodies** (Number of bodies in simulation)
- **BH Theta** (Theta value for Barnes-Hut algo. Higher value make the simulation run faster, but less accurate)
- **Donut Start** (Init bodies in a "Donut" formation instead of a square)
- **Initial Velocity** (Set body init velocity when in Donut Start)



### Resources

https://arborjs.org/docs/barnes-hut

https://github.com/pjankiewicz/nbody

https://github.com/awerries/gravity/blob/main/src/bhtree.rs

https://www.youtube.com/watch?v=L9N7ZbGSckk

https://www.youtube.com/watch?v=nZHjD3cI-EU

https://www.youtube.com/watch?v=GjbKsOkN1Oc&t=240s
