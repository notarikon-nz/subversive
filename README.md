# Subversive - Syndicate-Inspired Tactical Game

A quick & dirty 2D top-down squad-based tactical stealth game built with Rust and Bevy, inspired by the classic Syndicate series.

## Controls

| Input | Action |
|-------|--------|
| Space | Pause/Resume |
| Left Click | Select agent |
| Right Click | Move |
| N | Neurovector targeting |
| F | Combat targeting |
| E | Interact with terminals |
| WASD | Camera |
| I | Inventory |
| F3 | Toggle FPS |
| F5 | Save game (global map) |

## Gameplay

- Control 3 agents (green squares)
- Use neurovector to control civilians (yellow → purple when controlled)
- Avoid enemy vision cones (red square with yellow cone)
- Access terminals: Red (objectives), Blue (equipment), Green (intel)
- Agent progression with experience and levels
- Persistent save/load system

## Build

```bash
cargo run
```

Linux requirements:
```bash
sudo apt install build-essential pkg-config libasound2-dev libudev-dev
```

## Architecture

**Optimized Modular Structure:**
```
src/
├── main.rs              # Clean setup & scene loading
├── core/
│   ├── mod.rs          # Components & resources
│   ├── events.rs       # Minimal event system
│   ├── scenes.rs       # JSON scene system
│   └── save.rs         # Save/load persistence
└── systems/            # Focused single-purpose systems
    ├── input.rs        # Clean action handling
    ├── movement.rs     # Entity movement & pathfinding
    ├── selection.rs    # Agent selection
    ├── neurovector.rs  # Mind control targeting
    ├── interaction.rs  # Terminal access
    ├── combat.rs       # Attack system with health bars
    ├── camera.rs       # Simple camera movement
    ├── mission.rs      # Mission logic & post-mission
    └── ui.rs          # Visual feedback & gizmos
```

**Key Features:**
- JSON scene system for data-driven missions
- Entity pooling for performance
- Auto-save progression system
- Component composition over monolithic structures
- Proper Rust patterns with consistent error handling

**Engine:** [Bevy 0.14 ECS](https://bevy.org/) with [Rapier2D physics](https://rapier.rs/)