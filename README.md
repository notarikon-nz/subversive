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
| R | Reload Weapon |
| WASD | Camera movement |
| I | Inventory |
| F3 | Toggle FPS |
| F4 | Toggle GOAP AI Debug |
| F5 | Quick Save (in mission) |
| F8 | Quick Load (in mission) |
| F5 | Save game (global map) |

## Gameplay

- Select and command up to 3 agents simultaneously
- Use shift+click or drag boxes to build your squad
- Selected agents move together maintaining formation
- Use neurovector to control civilians (yellow → purple when controlled)
- Avoid enemy vision cones and detection - enemies will investigate sounds
- Access terminals: Red (objectives), Blue (equipment), Green (intel)
- Agent progression with experience, levels, and persistent save system

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
    ├── scenes.rs       # JSON scene system
    ├── save.rs         # Save/load persistence
    ├── quicksave.rs    # Mission quicksave system
    └── ui.rs           # Visual feedback & gizmos
```

**Key Features:**
- Squad-based multi-agent selection and control
- Enemy AI with sight, sound detection, and GOAP
- JSON scene system for data-driven missions
- Entity pooling for performance
- Audio system with context-aware sound effects
- Sprite system with graceful fallbacks
- Auto-save progression system
- Mission quicksave/quickload for tactical experimentation
- Reactive UI with proper state management

**Engine:** [Bevy 0.14 ECS](https://bevy.org/) with [Rapier2D physics](https://rapier.rs/)

---

**Current GOAP Actions**

Offensive:
- patrol, return_to_patrol, calm_down
- investigate, search_area
- attack, move_to_target, flank_target
Defensive:
- take_cover, retreat, reload, tactical_reload
Support:
- call_for_help, use_medkit
Advanced Tactical:
- throw_grenade, activate_alarm
- find_better_cover, suppressing_fire, fighting_withdrawal