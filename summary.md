# Subversive - Source Code Structure

## Overview
Subversive is a game built using the Bevy game engine in Rust. The codebase follows a modular architecture with distinct systems and core components.

## Directory Structure

### Main Entry Point (`main.rs`)
- Initializes the Bevy app with core plugins and resources
- Sets up game states and systems
- Configures physics, input management, and UI
- Orchestrates mission, combat, and environmental systems

### Core (`core/`)
- `agent_upgrades.rs`: Handles agent enhancement systems
- `attachments.rs`: Weapon attachment system
- `audio.rs`: Sound management system
- `cities.rs`: City/location management
- `collision_groups.rs`: Physics collision group definitions
- `components.rs`: Core ECS components
- `config.rs`: Game configuration management
- `despawn.rs`: Entity cleanup system
- `entities.rs`: Entity definitions and management
- `events.rs`: Game event definitions
- `factions.rs`: Faction system and relationships
- `fonts.rs`: Font resource management
- `game_state.rs`: Game state management
- `goap.rs`: Goal-Oriented Action Planning AI system
- `hackable.rs`: Hacking mechanics
- `input.rs`: Input handling system
- `lore.rs`: Game lore management
- `missions.rs`: Mission system
- `research.rs`: Research and progression system
- `resources.rs`: Resource management
- `scene_cache.rs`: Scene loading and caching
- `spawn_damage_text.rs`: Combat feedback system
- `sprites.rs`: Sprite resource management
- `weapons.rs`: Weapon system

### Systems (`systems/`)
Primary game systems organized by functionality:

#### Combat & AI
- `ai.rs`: AI behavior systems
- `combat.rs`: Combat mechanics
- `cover.rs`: Cover system
- `formations.rs`: Unit formation management
- `projectiles.rs`: Projectile physics and effects
- `weapon_swap.rs`: Weapon switching mechanics

#### Environment & Simulation
- `area_control.rs`: Territory control mechanics
- `day_night.rs`: Day/night cycle system
- `explosions.rs`: Explosion effects and damage
- `power_grid.rs`: Infrastructure management
- `urban_simulation.rs`: City simulation systems
- `vehicles.rs`: Vehicle mechanics

#### UI & Interaction
- `message_window.rs`: UI messaging system
- `scanner.rs`: Scanning mechanics
- `selection.rs`: Unit selection system
- `ui/`: User interface components

#### Mission & Game Flow
- `mission.rs`: Mission management
- `panic_spread.rs`: Morale and panic mechanics
- `quicksave.rs`: Save system
- `save.rs`: Game state persistence
- `scenes.rs`: Scene management

## Key Features

### AI and Behavior Systems
- Goal-Oriented Action Planning (GOAP)
- Sound detection and alert systems
- Morale and civilian behavior simulation
- Formation movement

### Combat Mechanics
- Weapon systems with attachments
- Cover and suppression mechanics
- Projectile physics
- Area control and damage systems

### Environment
- Urban simulation with civilian routines
- Day/night cycle
- Vehicle systems
- Power grid and infrastructure

### Game Systems
- Mission management
- Research and progression
- Save/load functionality
- Scene management with caching

### UI and Feedback
- Health bars and status indicators
- Message and notification systems
- Scanner interface
- Inventory management

## Technical Implementation
The game uses Bevy's ECS (Entity Component System) architecture extensively, with systems organized by functionality and run conditions based on game state. Physics is handled through the Bevy Rapier 2D plugin, and input through the Leafwing Input Manager.
