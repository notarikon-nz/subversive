use bevy::prelude::*;
use crate::core::game_state::{GlobalData, AlertLevel};

#[derive(Event)]
pub struct TerritoryControlEvent {
    pub city_id: String,
    pub event_type: TerritoryEventType,
}

#[derive(Debug)]
pub enum TerritoryEventType {
    ControlEstablished,
    ControlLost,
    ResistanceRising,
    TaxRateChanged(f32),
}

pub fn territory_event_system(
    mut events: EventReader<TerritoryControlEvent>,
    mut global_data: ResMut<GlobalData>,
) {
    for event in events.read() {
        match event.event_type {
            TerritoryEventType::ControlEstablished => {
                info!("Territory control established in {}", event.city_id);
            }
            TerritoryEventType::ControlLost => {
                warn!("Lost control of {}", event.city_id);
                global_data.cities_progress.get_city_state_mut(&event.city_id).alert_level = AlertLevel::Red;
            }
            TerritoryEventType::ResistanceRising => {
                warn!("Resistance rising in {}", event.city_id);
            }
            TerritoryEventType::TaxRateChanged(rate) => {
                info!("Tax rate in {} changed to {:.1}%", event.city_id, rate * 100.0);
            }
        }
    }
}
