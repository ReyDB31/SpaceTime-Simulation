# Spacetime Fabric (Rust)

A lightweight, real-time 2D spacetime-curvature visualization built with Rust + Macroquad.
The scene is rendered by a custom GLSL fragment shader: planets are procedural shaded spheres,
and each planet's mass bends the perspective grid around it.

## Run

1. Install Rust from https://rustup.rs/
2. Open a terminal in this folder.
3. Run:

   ```powershell
   cargo run --release
   ```

Cargo downloads Macroquad on the first run. No external textures, models, or asset files are needed.

## Controls

- **Left-click empty space** — add a planet
- **Left-click a planet** — select it
- **Right-click a planet** — remove it
- **+ / -** — increase / decrease selected planet's mass
- **N** — add a planet at a random location
- **Delete** — remove selected planet
- **R** — reset the scene
- **H** — show / hide controls

Up to 8 planets can be active at once.

## Notes

This is an educational visual model, not a numerical general-relativity solver. The grid deformation
is a stylized shader approximation of gravitational curvature; it is not a physically exact
spacetime metric or orbital simulation. The shader and procedural planet surface are included in
`src/main.rs` so you can modify them directly.
