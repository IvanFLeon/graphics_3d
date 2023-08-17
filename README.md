# A simple and naive 3D graphics library written in Rust

This is an experimental project to understand what goes behind writing a simple graphics API.

This API provides a few geometries, transformations and color functions that allow you to draw 2D (and possibly 3D) objects to the screen.

- The renderer is built with wgpu.
- The math is done with glam.
- The window is handled by winit.

## How does it work?

It is very simple, we use a tree for storing the geometry primitives, transformations and colors.

Transformations and colors are of the done through `push()` and `pop()`, so you can imagine how a push would create a new leaf node, and a pop would return you back to the parent node.
