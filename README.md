# Subversive - Syndicate-Inspired Tactical Game

A quick & dirty 2D top-down squad-based tactical stealth game built with Rust and Bevy, inspired by the classic Syndicate series.

## Controls

| Input | Action |
|-------|--------|
| Space | Pause/Resume |
| Left Click | Select agent |
| Shift+Left Click | Add/Remove agent from squad |
| Drag | Select multiple agents in box |
| Right Click | Move selected agents |
| N | Neurovector targeting |
| F | Combat targeting |
| E | Interact with terminals |
| WASD | Camera movement |
| I | Inventory |
| F3 | Toggle FPS |
| F5 | Save game (global map) |

## Gameplay

- **Squad Control**: Select and command up to 3 agents simultaneously
- **Multi-selection**: Use shift+click or drag boxes to build your squad
- **Formation movement**: Selected agents move together maintaining formation
- Use neurovector to control civilians (yellow → purple when controlled)
- Avoid enemy vision cones and detection - enemies will investigate sounds
- Access terminals: Red (objectives), Blue (equipment), Green (intel)
- Agent progression with experience, levels, and persistent save system

## Squad Mechanics

- **Individual Selection**: Left-click to select a single agent
- **Multi-Selection**: Shift+left-click to add/remove agents from your squad
- **Box Selection**: Click and drag to select all agents within the area
- **Group Commands**: Right-click to move all selected agents together
- **Formation Indicators**: Visual lines show your squad formation
- **Smart Targeting**: Special abilities use the primary selected agent

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
│   ├── audio.rs        # Event-driven audio system
│   ├── sprites.rs      # Sprite management
│   └── ai.rs           # Enemy AI behavior
└── systems/            # Focused single-purpose systems
    ├── input.rs        # Clean action handling
    ├── movement.rs     # Entity movement & pathfinding
    ├── selection.rs    # Multi-agent selection system
    ├── neurovector.rs  # Mind control targeting
    ├── interaction.rs  # Terminal access
    ├── combat.rs       # Attack system with health bars
    ├── camera.rs       # Simple camera movement
    ├── mission.rs      # Mission logic & post-mission
│   ├── scenes.rs       # JSON scene system
│   ├── save.rs         # Save/load persistence
    └── ui.rs           # Visual feedback & gizmos
```

**Key Features:**
- Squad-based multi-agent selection and control
- Enemy AI with sight, sound detection, and state machines
- JSON scene system for data-driven missions
- Entity pooling for performance
- Audio system with context-aware sound effects
- Sprite system with graceful fallbacks
- Auto-save progression system
- Reactive UI with proper state management

**Engine:** [Bevy 0.14 ECS](https://bevy.org/) with [Rapier2D physics](https://rapier.rs/)