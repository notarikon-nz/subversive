# Subversive - Source Code Structure

## Overview
Subversive is a cyberpunk tactical game built using the Bevy game engine in Rust. The codebase follows a modular architecture with distinct systems for urban simulation, combat, research, and environmental effects.

## Recent Features (v0.2.12 - v0.2.16)

### v0.2.16 - Isometric View Update
- Isometric camera system and tilemap implementation
- Depth sorting for entities in isometric view
- Camera zoom levels and movement adjustments
- Scene loading optimization for isometric perspective

### v0.2.15 - Enhanced Inventory
- Grid-based inventory system
- Loadout management and caching
- Item action handling
- Hotkey system for loadouts

### v0.2.14 - World Scanning
- Advanced scanning mechanics for intelligence gathering
- Scanner energy management
- Scan overlay visualization
- Entity scanning and information display

### v0.2.13 - Weather System
- Dynamic weather states (Clear, Light Rain, Heavy Rain, Snow)
- Weather effects tied to city climate and traits
- Particle system for weather visualization
- Gameplay impacts (visibility, movement)
- Weather overlay system

### v0.2.12 - Research and Scientists
- Research facility system
- Scientist recruitment and management
- Research progression and sabotage mechanics
- Loyalty and productivity systems

## Core Systems

### Urban Simulation
- Civilian daily routines and behaviors
- Urban zones (residential, commercial, industrial)
- Traffic system with various vehicle types
- Public transit and foot traffic simulation

### Traffic System
- Advanced vehicle AI with flow fields
- Emergency response vehicles
- Military convoys
- Traffic light management
- Vehicle collisions and damage
- Road network and pathfinding

### Research and Development
- Technology tree with multiple categories
- Scientist management and specializations
- Research facility mechanics
- Espionage and sabotage options
- Benefits system for unlocks

### Weather and Environment
- City-specific weather patterns
- Dynamic particle effects
- Visual overlays
- Gameplay impact system
- Season and climate zone effects

### Hacking and Infrastructure
- Financial systems (ATMs, banking network)
- Security systems (cameras, turrets)
- Power grid management
- Access control systems
- Network scanning and infiltration

## Technical Implementation

### Core Architecture
- Entity Component System (ECS) using Bevy
- State-based game flow
- Event-driven systems
- Resource management
- Scene caching

### Physics and Movement
- Rapier 2D physics integration
- Pathfinding systems
- Collision handling
- Movement controls

### UI and Feedback
- Egui integration for interface
- Status indicators
- Minimap system
- Interaction prompts
- Debug visualization tools

### Performance Optimizations
- Entity pooling
- Scene caching
- Particle system optimization
- Grid-based spatial queries
- Timed system execution

## Asset Management
- Sprites and animations
- Audio resources
- Scene definitions
- Configuration files
- Urban layout data

## Development Tools
- Debug systems
- Testing scenarios
- Performance monitoring
- Scene editing
- Configuration management
