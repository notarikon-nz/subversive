// src/core/goap_extensions.rs - Examples of extending the GOAP system
use bevy::prelude::*;
use crate::core::goap::*;
use crate::core::*;

// === EXTENDED WORLD STATES ===
// Add these to your WorldKey enum:
/*
pub enum WorldKey {
    // Existing keys...
    
    // Equipment states
    HasArmor,
    ArmorDamaged,
    HasMedKit,
    IsInjured,
    
    // Environmental states
    InCover,
    DoorLocked,
    AlarmActive,
    
    // Communication states
    RadioWorking,
    BackupCalled,
    TeamNearby,
    
    // Tactical states
    FlankingPosition,
    HighGround,
    GoodVantagePoint,
}
*/

// === EQUIPMENT COMPONENT ===
#[derive(Component)]
pub struct Equipment {
    pub armor: Option<ArmorType>,
    pub has_medkit: bool,
    pub radio_working: bool,
    pub ammo: u32,
    pub max_ammo: u32,
}

#[derive(Debug, Clone)]
pub enum ArmorType {
    Light,
    Heavy,
    Tactical,
}

// === EXTENDED ACTIONS ===
impl GoapAgent {
    pub fn add_advanced_actions(&mut self) {
        let advanced_actions = vec![
            // Take cover action
            GoapAction {
                name: "take_cover",
                cost: 1.5,
                preconditions: hashmap![
                    WorldKey::HasTarget => true,
                    WorldKey::InCover => false
                ],
                effects: hashmap![
                    WorldKey::InCover => true
                ],
                action_type: ActionType::TakeCover,
            },
            
            // Use medkit action
            GoapAction {
                name: "use_medkit",
                cost: 3.0,
                preconditions: hashmap![
                    WorldKey::IsInjured => true,
                    WorldKey::HasMedKit => true
                ],
                effects: hashmap![
                    WorldKey::IsInjured => false,
                    WorldKey::HasMedKit => false
                ],
                action_type: ActionType::UseMedKit,
            },
            
            // Call for backup
            GoapAction {
                name: "call_backup",
                cost: 2.0,
                preconditions: hashmap![
                    WorldKey::HasTarget => true,
                    WorldKey::RadioWorking => true,
                    WorldKey::BackupCalled => false
                ],
                effects: hashmap![
                    WorldKey::BackupCalled => true
                ],
                action_type: ActionType::CallBackup,
            },
            
            // Flank target
            GoapAction {
                name: "flank_target",
                cost: 4.0,
                preconditions: hashmap![
                    WorldKey::HasTarget => true,
                    WorldKey::TargetVisible => true,
                    WorldKey::FlankingPosition => false
                ],
                effects: hashmap![
                    WorldKey::FlankingPosition => true,
                    WorldKey::AtTarget => true
                ],
                action_type: ActionType::FlankTarget,
            },
        ];
        
        self.available_actions.extend(advanced_actions);
    }
    
    pub fn add_advanced_goals(&mut self) {
        let advanced_goals = vec![
            Goal {
                name: "survive_encounter",
                priority: 15.0, // Higher than eliminate_threat
                desired_state: hashmap![
                    WorldKey::IsInjured => false,
                    WorldKey::InCover => true,
                    WorldKey::BackupCalled => true
                ],
            },
            
            Goal {
                name: "tactical_advantage",
                priority: 7.0,
                desired_state: hashmap![
                    WorldKey::FlankingPosition => true,
                    WorldKey::InCover => true
                ],
            },
        ];
        
        self.goals.extend(advanced_goals);
    }
}

// === DYNAMIC BEHAVIOR SYSTEM ===
#[derive(Component)]
pub struct BehaviorProfile {
    pub aggression: f32,      // 0.0 = cautious, 1.0 = aggressive
    pub intelligence: f32,    // 0.0 = basic, 1.0 = tactical genius
    pub teamwork: f32,        // 0.0 = lone wolf, 1.0 = team player
    pub fear_threshold: f32,  // Health % where they start defensive behaviors
}

impl Default for BehaviorProfile {
    fn default() -> Self {
        Self {
            aggression: 0.5,
            intelligence: 0.5,
            teamwork: 0.5,
            fear_threshold: 0.3,
        }
    }
}

impl BehaviorProfile {
    pub fn adjust_action_costs(&self, action: &mut GoapAction) {
        match action.name {
            "attack" => {
                action.cost = 1.0 / (1.0 + self.aggression); // More aggressive = cheaper attacks
            },
            "take_cover" => {
                action.cost = 1.0 + self.aggression; // More aggressive = more expensive cover
            },
            "call_backup" => {
                action.cost = 2.0 / (1.0 + self.teamwork); // More teamwork = cheaper backup calls
            },
            "flank_target" => {
                action.cost = 4.0 / (1.0 + self.intelligence); // Smarter = better tactics
            },
            _ => {}
        }
    }
    
    pub fn adjust_goal_priorities(&self, goal: &mut Goal) {
        match goal.name {
            "eliminate_threat" => {
                goal.priority = 10.0 * (1.0 + self.aggression);
            },
            "survive_encounter" => {
                goal.priority = 15.0 * (2.0 - self.aggression); // Less aggressive = higher survival priority
            },
            "tactical_advantage" => {
                goal.priority = 7.0 * (1.0 + self.intelligence);
            },
            _ => {}
        }
    }
}

// === SQUAD COORDINATION SYSTEM ===
#[derive(Component)]
pub struct SquadMember {
    pub squad_id: u32,
    pub role: SquadRole,
}

#[derive(Debug, Clone)]
pub enum SquadRole {
    Leader,
    Support,
    Scout,
    Heavy,
}

#[derive(Resource, Default)]
pub struct SquadCoordination {
    pub squads: std::collections::HashMap<u32, SquadData>,
}

pub struct SquadData {
    pub members: Vec<Entity>,
    pub leader: Option<Entity>,
    pub current_objective: Option<SquadObjective>,
    pub formation: Formation,
}

#[derive(Debug, Clone)]
pub enum SquadObjective {
    Patrol,
    Hunt { target: Entity },
    Defend { position: Vec2 },
    Retreat { rally_point: Vec2 },
}

#[derive(Debug, Clone)]
pub enum Formation {
    Line,
    Wedge,
    Circle,
    Scattered,
}

// === EXAMPLE: GRENADE THROWING BEHAVIOR ===
pub fn add_grenade_behavior(goap_agent: &mut GoapAgent) {
    // Add grenade-related world states to WorldKey enum first:
    // HasGrenade, TargetGrouped, SafeThrowDistance
    
    goap_agent.available_actions.push(GoapAction {
        name: "throw_grenade",
        cost: 2.0,
        preconditions: hashmap![
            WorldKey::HasTarget => true,
            // WorldKey::HasGrenade => true,
            // WorldKey::SafeThrowDistance => true,
        ],
        effects: hashmap![
            WorldKey::HasTarget => false, // Assume grenade eliminates threat
            // WorldKey::HasGrenade => false,
        ],
        action_type: ActionType::ThrowGrenade,
    });
}

// === DYNAMIC ACTION EXECUTION ===
pub fn execute_extended_goap_action(
    action: &GoapAction,
    enemy_entity: Entity,
    enemy_transform: &Transform,
    mut commands: &mut Commands,
    action_events: &mut EventWriter<ActionEvent>,
    audio_events: &mut EventWriter<AudioEvent>,
) {
    match action.name {
        "take_cover" => {
            // Find nearest cover point and move there
            let cover_position = find_nearest_cover(enemy_transform.translation.truncate());
            action_events.write(ActionEvent {
                entity: enemy_entity,
                action: Action::MoveTo(cover_position),
            });
            info!("Enemy {} taking cover", enemy_entity.index());
        },
        
        "use_medkit" => {
            // Heal the enemy
            if let Some(mut health_commands) = commands.get_entity(enemy_entity) {
                // You'd need to add a healing component or event here
                info!("Enemy {} using medkit", enemy_entity.index());
            }
        },
        
        "call_backup" => {
            // Trigger backup spawning or alert other enemies
            audio_events.write(AudioEvent {
                sound: AudioType::Alert,
                volume: 1.0,
            });
            info!("Enemy {} calling for backup", enemy_entity.index());
        },
        
        "flank_target" => {
            // Calculate flanking position and move there
            let flank_position = calculate_flanking_position(enemy_transform.translation.truncate());
            action_events.write(ActionEvent {
                entity: enemy_entity,
                action: Action::MoveTo(flank_position),
            });
            info!("Enemy {} attempting to flank", enemy_entity.index());
        },
        
        _ => {
            // Handle other actions or fall back to default behavior
        }
    }
}

// === UTILITY FUNCTIONS ===
fn find_nearest_cover(position: Vec2) -> Vec2 {
    // Simple example - in a real game you'd use proper cover detection
    // For now, just move to a position offset from current location
    position + Vec2::new(-50.0, 0.0)
}

fn calculate_flanking_position(enemy_pos: Vec2) -> Vec2 {
    // Simple flanking logic - move to a side position
    // In a real implementation, you'd calculate based on target position and environment
    enemy_pos + Vec2::new(100.0, 50.0)
}

// === REACTIVE BEHAVIOR SYSTEM ===
pub fn reactive_goap_system(
    mut goap_query: Query<(&mut GoapAgent, &BehaviorProfile, &Health), With<Enemy>>,
    time: Res<Time>,
) {
    for (mut goap_agent, behavior_profile, health) in goap_query.iter_mut() {
        let health_ratio = health.0 / 100.0; // Assuming max health is 100
        
        // Adjust behavior based on health
        if health_ratio < behavior_profile.fear_threshold {
            // Increase survival goal priority when injured
            for goal in &mut goap_agent.goals {
                if goal.name == "survive_encounter" {
                    goal.priority = 20.0; // Highest priority when scared
                }
            }
            
            // Make defensive actions cheaper
            for action in &mut goap_agent.available_actions {
                if action.name == "take_cover" || action.name == "call_backup" {
                    action.cost *= 0.5; // Half cost when injured
                }
            }
            
            // Force replanning
            goap_agent.abort_plan();
        }
    }
}

// === EXAMPLE INTEGRATION SYSTEM ===
pub fn behavior_profile_system(
    mut goap_query: Query<(&mut GoapAgent, &BehaviorProfile), (With<Enemy>, Changed<BehaviorProfile>)>,
) {
    for (mut goap_agent, behavior_profile) in goap_query.iter_mut() {
        // Apply behavior profile to all actions and goals
        for action in &mut goap_agent.available_actions {
            behavior_profile.adjust_action_costs(action);
        }
        
        for goal in &mut goap_agent.goals {
            behavior_profile.adjust_goal_priorities(goal);
        }
        
        // Force replanning with new costs/priorities
        goap_agent.abort_plan();
    }
}