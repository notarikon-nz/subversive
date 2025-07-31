# Subversive - Syndicate-Inspired Tactical Game

A 2D squad-based tactical stealth game built with Rust and Bevy, inspired by the classic Syndicate series and Satellite Reign.

## **Key Features**

### **Tactical Squad Combat**
- Multi-agent selection and control with formation systems
- Enemy AI with sight, sound detection, and GOAP behavioral system
- Cover-based combat with dynamic positioning
- Weapon specialization system with multiple weapon types (pistols to plasma guns)
- Advanced area control and suppression mechanics, including dynamic explosion system

### **World Scan System**
- Satellite Reign-inspired intelligence gathering
- 5-mode scanning: Infrastructure, Security, Financial, Personnel, All Systems
- Energy-based scanner management with upgrades
- Network topology visualization showing power grid and security connections
- Real-time threat assessment with 5-tier classification system

### **Infrastructure Hacking**
- Power grid manipulation affecting entire districts
- Hackable devices: cameras, turrets, doors, ATMs, terminals
- Network dependency mapping (disable power station → lights go dark)
- Security bypass through access cards or hacking tools
- Financial network infiltration with banking data extraction

### **Advanced AI & Emergent Gameplay**
- GOAP (Goal-Oriented Action Planning) enemy behavior
- Dynamic civilian crowds with panic propagation
- Police escalation system with 5 threat levels
- Urban simulation with work/shopping/residential zones
- Neurovector mind control capabilities

### **Living Cyberpunk World**
- Day/night cycle affecting visibility and NPC behavior
- Dynamic weather system with gameplay impact
- Traffic simulation with emergency response
- Research facilities with recruitable scientists
- Procedural mission content with branching objectives

### **Quality of Life Features**
- JSON scene system for data-driven missions
- Entity pooling for 60+ FPS performance
- Mission quicksave/quickload for tactical experimentation
- Auto-save progression system
- Comprehensive UI with egui integration
- Audio system with context-aware sound effects

### **Progression & Customization**
- Research tree with 20+ upgrades affecting gameplay
- Weapon attachment system with stat modifications
- Agent specialization and experience system
- Cybernetic enhancement integration
- Global meta-progression between missions

### **Technical Excellence**
- Built on Bevy 0.16.1 ECS architecture
- Rapier2D physics with collision optimization
- Pathfinding system with dynamic obstacle avoidance
- Modular system design with hot-reload capabilities
- Memory-efficient sprite management with fallbacks

---

## **Current GOAP Actions**

**Offensive:**
- patrol, return_to_patrol, calm_down
- investigate, search_area
- attack, move_to_target, flank_target

**Defensive:**
- take_cover, retreat, reload, tactical_reload

**Support:**
- call_for_help, use_medkit

**Advanced Tactical:**
- throw_grenade, activate_alarm
- find_better_cover, suppressing_fire, fighting_withdrawal

**Infrastructure:**
- hack_device, access_terminal, manipulate_power_grid
- bypass_security, extract_financial_data

---

## **Controls**

**Core:**
- WASD: Camera movement
- Mouse: Agent selection and commands
- Space: Pause
- E: Interact with objects/terminals

**World Scanner:**
- Tab: Cycle scan modes
- Enter: Perform scan
- O: Toggle overlay visibility
- Q: Basic scanner window

**Combat:**
- F: Combat mode
- R: Reload
- I: Inventory
- N: Neurovector control

---

**Engine:** [Bevy 0.16.1 ECS](https://bevy.org/) with [Rapier2D physics](https://rapier.rs/)