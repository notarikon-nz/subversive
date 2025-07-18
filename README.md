# Subversive - Syndicate-Inspired Tactical Game

A quick and dirty 2D top-down squad-based tactical stealth game built with Rust and Bevy, inspired by the classic Syndicate series.

## Current Features (Playable MVP)

### Core Mechanics
- **Real-time with pause**: Press `Space` to pause/resume time
- **Agent movement**: Right-click to move agents
- **Camera control**: WASD or arrow keys to move camera
- **Basic selection**: Left-click to select units (visual feedback)
- **Mission structure**: Simple infiltration objective

### Neurovector Mind Control

- Targeting mode: Press N to enter neurovector targeting
- Range indicator: Blue circle shows neurovector range (red when on cooldown)
- Target selection: Click on highlighted civilians to control them
- Visual feedback: Purple lines connect agents to controlled civilians
- Controlled movement: Controlled civilians (purple) follow movement orders
- Cooldown system: 5-second cooldown between neurovector uses

### Interactive Terminal System

- Color-coded terminals: Red (critical), Blue (secondary), Green (optional)
- Proximity detection: Get close to terminals to see interaction prompts
- 'E' to interact: Hold position while accessing terminals
- Progress system: Visual progress bars show interaction timing
- Loot rewards: Terminals provide currency, intel, and skill matrices

### What You'll See
- **3 Green Agents**: Your controllable squad members
- **5 Yellow Civilians**: Potential neurovector targets
- **1 Red Enemy**: Patrolling guard with detection range
- **1 Purple Objective**: Target infiltration point

## Controls

| Key/Mouse | Action |
|-----------|--------|
| `Space` | Pause/Resume |
| `Right Click` | Move selected agents |
| `Left Click` | Select agent/unit |
| `WASD` / `Arrow Keys` | Move camera |

## Architecture Overview

### Core Components
- **Agent**: Player-controlled units with health, skills, cybernetics
- **Civilian**: NPCs that can be controlled via neurovector
- **Enemy**: Hostile units with patrol routes and detection
- **Mission Objectives**: Win conditions and targets

### Key Systems
- **Movement System**: Pathfinding and agent positioning
- **Pause System**: Time control for tactical planning
- **Visibility System**: Line of sight and detection mechanics
- **Alert System**: Security response escalation

### Game States
- **Mission State**: Active, Paused, Complete, Failed
- **Alert Levels**: Green → Yellow → Orange → Red

## Next Development Phases

### Phase 2 - Tactical Features (Next Up)
- [X] Mind control
- [X] Interactive terminal system with color-coding*
- [X] Stealth detection mechanics (enemy vision cones)
- [ ] Basic combat system
- [ ] Equipment and cybernetics

* Terminal systems can hide both mission-relevant and lore data - colour-coding allows the player to know which is which, in case the lore is not of interest.

### Phase 3 - Strategy Layer
- [ ] Global map and region control
- [ ] Agent progression and skill matrices
- [ ] Resource management
- [ ] Mission generation

### Phase 4 - Polish
- [ ] Advanced AI behaviors
- [ ] Animations and effects
- [ ] Audio system
- [ ] UI/UX improvements

## Technical Notes

- **Engine**: Bevy 0.14 with ECS architecture
- **Physics**: Rapier2D for collision and movement
- **Input**: Leafwing Input Manager for flexible controls
- **Performance**: Optimized for 50+ entities per mission

## Extending the Game

The framework is designed for rapid iteration. Key extension points:

1. **Add new agent actions** in `components.rs` (`AgentAction` enum)
2. **Create new mission types** in `systems.rs` (`spawn_test_mission`)
3. **Implement cybernetics** in `components.rs` (`CyberneticType`)
4. **Add new objectives** in `components.rs` (`ObjectiveType`)

## Debug Features

- Physics debug rendering (collision shapes visible)
- Entity count logging
- Alert level state changes
- Mission timer in console

## Linux Builds

```
sudo apt-get update
sudo apt-get install build-essential pkg-config libasound2-dev libudev-dev libx11-dev libxi-dev libgl1-mesa-dev
```