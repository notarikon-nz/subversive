# Subversive - Syndicate-Inspired Tactical Game

A quick and dirty 2D top-down squad-based tactical stealth game built with Rust and Bevy, inspired by the classic Syndicate series.

### What You'll See
- **3 Green Agents**: Your controllable squad members with neurovector capabilities
- **5 Yellow Civilians**: Potential neurovector targets (turn purple when controlled)
- **1 Red Enemy**: Patrolling guard with **visible vision cone** and detection range
- **1 Purple Objective**: Target infiltration point
- **3 Terminals**: Red (mission critical), Blue (cybernetics), Green (intel/lore)
- **Vision Cones**: Yellow/orange/red areas showing enemy sight ranges

## Controls

| Key/Mouse | Action |
|-----------|--------|
| `Space` | Pause/Resume |
| `Left Click` | Select agent |
| `Right Click` | Move selected agents/controlled civilians |
| `N` | Enter/Exit neurovector targeting mode |
| `E` | Interact with nearby terminals |
| `I` | Open/Close inventory for selected agent |
| `Escape` | Cancel neurovector targeting, terminal interaction, or close inventory |
| `WASD` / `Arrow Keys` | Move camera |

### Neurovector Usage
1. **Select an agent** (left-click) - you'll see a blue range circle
2. **Press N** to enter targeting mode - valid targets get yellow highlights
3. **Click a civilian** within range to control them
4. **Use right-click** to move both agents and controlled civilians together
### Equipment Management
1. **Access terminals** to collect weapons, tools, cybernetics, and intel
2. **Press 'I'** to open the inventory panel for your selected agent
3. **View organized categories** - each item type has its own section
4. **Watch for notifications** - green popups show newly acquired items
5. **Track currency** - credits earned from terminal rewards
6. **Read intel documents** - lore and mission information stored automatically

### Stealth Gameplay
1. **Watch enemy vision cones** - yellow areas show where enemies can see
2. **Stay out of sight** when moving to terminals
3. **Monitor detection circles** - red circles above enemies show if you're being spotted
4. **Use pause strategically** - plan movements when enemies face away
5. **Controlled civilians** can also be detected, adding complexity

### Terminal Interaction
1. **Move agent near a terminal** - you'll see a colored interaction range
2. **Press E** to start accessing the terminal
3. **Stay in position** while the progress bar fills
4. **Collect rewards** automatically when interaction completes
5. **Use Escape** to cancel interaction if needed

Color coding:
- **Red terminals**: Mission-critical objectives  
- **Blue terminals**: Valuable cybernetics and skill matrices
- **Green terminals**: Optional lore and intel

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

**Next Milestone**: Basic combat system and equipment effects

### Phase 2 - Tactical Features (Next Up)
- [X] Mind control
- [X] Interactive terminal system with color-coding*
- [X] Stealth detection mechanics (enemy vision cones)
- [x] Equipment inventory and reward management
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
