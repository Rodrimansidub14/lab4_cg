# Celestial Bodies Shader Renderer

A Rust-based graphics program that renders various celestial bodies like stars, planets, moons, and comets using custom shaders. This project uses noise functions to generate textures that simulate real-world surfaces, including craters, rocky terrain, and gas giants with banding.

## Features

- Realistic rendering of stars, rocky planets, gas giants, and moons.
- Adjustable shaders for each celestial type, including noise-based textures for surfaces and rings.
- Basic light direction simulation for added realism.
- Supports rotating the camera around celestial objects and zooming in/out.

##Controls
- Zoom in: PRESS KEY "Z"
- Zoom out: PRESS KEY "X"
- Next planet:Key "N"
- For specific planets: Keys 1-7
  

## Screenshots
- **Star**:
  ![image](https://github.com/user-attachments/assets/bd6c7b94-3611-4203-9f7b-be77231bb00d)

- **Rocky Planet**:
  ![image](https://github.com/user-attachments/assets/11ffdb06-0bbb-474e-968c-b51c7a966d69)

- **Gas Giant**:
  ![image](https://github.com/user-attachments/assets/bb55b40d-d74a-41b0-994f-26c89a6408b2)

- **Ringed Gas Giant**:
  ![image](https://github.com/user-attachments/assets/977cebfd-181e-4b14-8338-428b29f4ee94)

- **Rocky planet with moon**:
  ![image](https://github.com/user-attachments/assets/b12d12fd-9b8e-4939-b661-22a12d8fcda0)

- **Mars Like Rocky**:
  ![image](https://github.com/user-attachments/assets/641242c2-bb81-4381-b75d-bd8f700b3a26)

- **Comet**:
- ![image](https://github.com/user-attachments/assets/5e6f0800-4456-41d5-9b10-eea5ccca4e5a)

## Requirements

- Rust (latest stable version recommended)
- Cargo
- `minifb` crate for window and rendering
- `nalgebra_glm` for matrix and vector math
- `fastnoise_lite` crate for noise generation

To install Rust and Cargo, visit [rust-lang.org](https://www.rust-lang.org/).

## Installation

Clone this repository:

```bash
git clone https://github.com/your-username/celestial-bodies-renderer.git
cd celestial-bodies-renderer
