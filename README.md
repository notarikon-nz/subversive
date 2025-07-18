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

## Gameplay

- Control 3 agents (green squares)
- Use neurovector to control civilians (yellow → purple when controlled)
- Avoid enemy vision cones (red square with yellow cone)
- Access terminals: Red (objectives), Blue (equipment), Green (intel)

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
├── main.rs              # Clean setup & spawning
├── core/
│   ├── mod.rs          # Components & resources
│   └── events.rs       # Minimal event system
└── systems/            # Focused single-purpose systems
    ├── input.rs        # Clean action handling
    ├── movement.rs     # Entity movement & pathfinding
    ├── selection.rs    # Agent selection
    ├── neurovector.rs  # Mind control targeting
    ├── interaction.rs  # Terminal access
    ├── combat.rs       # Attack system with health bars
    ├── camera.rs       # Simple camera movement
    └── ui.rs          # Visual feedback & gizmos
```

**Key Improvements:**
- Component composition over monolithic structures
- ~60% smaller codebase with modular systems
- Proper Rust patterns with consistent error handling

**Engine:** Bevy 0.14 ECS with Rapier2D physics



