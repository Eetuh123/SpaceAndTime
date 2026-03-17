# Space and Time

## What is this?
Space and time visualizer built with Rust and wgpu, which displays how objects in space get distorted by different objects and shows their effect on the spacetime continuum.

## Architecture
| File | Purpose |
|------|---------|
| main.rs | Pipeline creation window interactions basically the machine what makes the whole thing run |
| camera.rs | Camera controls |
| texture.rs | Depth buffer |
| gravity.rs | Gravitys uniform and intesity |
| shader.wgsl | Shaders |

## Currently implemented
- Sphere
- Space and time grid
- Depth buffer
- Camera movement
- Basic interaction with grid and objects in space

## Planned
- Black hole distortion to the light
- Gravity interaction between objects

## Dependencies
| Crate | Why |
|-------|-----|
| wgpu | Modern way to interact with GPU and good rust implementation |
| winit | Allows easier interaction with window manager |
| glam | Math library of choice |
| bytemuck | Conversion for Vectors to bytes  |
| pollster | For running Async functions synchronously |

## Controls
| Key | Action |
|-----|--------|
| W/S | Z-axis |
| A/D | Roates around the center point |
| Shift + A/D | X-Axis |
| Shift + W/S | Y-Axis |
| Escape | Close |