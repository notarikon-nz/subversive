// ui_iced/singapore_map.rs - Singapore vector map rendering
use bevy::prelude::*;
use iced::{
    widget::{canvas, container, column, row, text, button, Canvas},
    Element, Length, Color, Point, Size, Vector,
};
use iced::widget::canvas::{Cache, Frame, Geometry, Path, Stroke, Fill, LineCap};
use crate::ui_iced::{Message, HubMsg};
use crate::core::*;
use crate::systems::ui::hub::singapore_map::*;

pub struct SingaporeMapCanvas {
    cache: Cache,
    singapore_map: SingaporeVectorMap,
    selected_district: Option<String>,
    hovered_district: Option<String>,
}

impl SingaporeMapCanvas {
    pub fn new(map: SingaporeVectorMap) -> Self {
        Self {
            cache: Cache::new(),
            singapore_map: map,
            selected_district: None,
            hovered_district: None,
        }
    }

    pub fn view(&self) -> Element<Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fixed(600.0))
            .into()
    }
}

impl canvas::Program<Message> for SingaporeMapCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: canvas::Cursor,
    ) -> Vec<Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            // Dark background
            frame.fill_rectangle(
                Point::ORIGIN,
                bounds.size(),
                Fill::from(Color::from_rgb(0.03, 0.05, 0.08)),
            );

            // Draw coastline
            draw_coastline(frame, &self.singapore_map, bounds);

            // Draw districts
            for (district_id, geometry) in &self.singapore_map.districts {
                draw_district(
                    frame,
                    &self.singapore_map,
                    bounds,
                    district_id,
                    geometry,
                    self.selected_district.as_ref() == Some(district_id),
                    self.hovered_district.as_ref() == Some(district_id),
                );
            }

            // Draw MRT lines
            draw_mrt_system(frame, &self.singapore_map, bounds);

            // Draw landmarks
            draw_landmarks(frame, &self.singapore_map, bounds);
        });

        vec![geometry]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: iced::Rectangle,
        cursor: canvas::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        match event {
            canvas::Event::Mouse(mouse_event) => {
                if let Some(position) = cursor.position_in(bounds) {
                    let world_pos = screen_to_world(position, bounds, &self.singapore_map);
                    
                    // Find district at position
                    for (district_id, geometry) in &self.singapore_map.districts {
                        if point_in_polygon(world_pos, &geometry.boundary) {
                            match mouse_event {
                                iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left) => {
                                    return (
                                        canvas::event::Status::Captured,
                                        Some(Message::Hub(HubMsg::SelectCity(district_id.clone()))),
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        (canvas::event::Status::Ignored, None)
    }
}

fn draw_coastline(frame: &mut Frame, map: &SingaporeVectorMap, bounds: iced::Rectangle) {
    if map.coastline.len() < 2 { return; }

    let path = Path::new(|builder| {
        let first = world_to_screen(map.coastline[0], bounds, map);
        builder.move_to(first);
        
        for &point in map.coastline.iter().skip(1) {
            builder.line_to(world_to_screen(point, bounds, map));
        }
    });

    frame.stroke(
        &path,
        Stroke::default()
            .with_width(2.0)
            .with_color(Color::from_rgb(0.27, 0.51, 0.71)),
    );
}

fn draw_district(
    frame: &mut Frame,
    map: &SingaporeVectorMap,
    bounds: iced::Rectangle,
    district_id: &str,
    geometry: &DistrictGeometry,
    selected: bool,
    hovered: bool,
) {
    if geometry.boundary.len() < 3 { return; }

    let path = Path::new(|builder| {
        let first = world_to_screen(geometry.boundary[0], bounds, map);
        builder.move_to(first);
        
        for &point in geometry.boundary.iter().skip(1) {
            builder.line_to(world_to_screen(point, bounds, map));
        }
        builder.close();
    });

    // Fill color
    let mut fill_color = geometry.control_color;
    if selected {
        fill_color = brighten_iced_color(fill_color, 1.3);
    } else if hovered {
        fill_color = brighten_iced_color(fill_color, 1.1);
    }

    frame.fill(&path, Fill::from(fill_color));

    // Border
    let stroke = match geometry.border_style {
        BorderStyle::Controlled => Stroke::default().with_width(2.0).with_color(Color::WHITE),
        BorderStyle::Contested => Stroke::default().with_width(2.0).with_color(Color::from_rgb(1.0, 1.0, 0.0)),
        BorderStyle::Corporate => Stroke::default().with_width(1.5).with_color(Color::from_rgba(0.5, 0.5, 0.5, 1.0)),
        BorderStyle::Neutral => Stroke::default().with_width(1.0).with_color(Color::from_rgba(0.3, 0.3, 0.3, 1.0)),
    };

    frame.stroke(&path, stroke);

    // Label
    let center = world_to_screen(geometry.center, bounds, map);
    frame.fill_text(iced::widget::canvas::Text {
        content: district_id.replace("_", " "),
        position: center,
        color: Color::WHITE,
        size: 12.0.into(),
        font: iced::Font::DEFAULT,
        horizontal_alignment: iced::alignment::Horizontal::Center,
        vertical_alignment: iced::alignment::Vertical::Center,
    });
}

fn draw_mrt_system(frame: &mut Frame, map: &SingaporeVectorMap, bounds: iced::Rectangle) {
    // Draw lines
    for line in map.mrt_system.lines.values() {
        if line.path.len() < 2 { continue; }

        let path = Path::new(|builder| {
            let first = world_to_screen(line.path[0], bounds, map);
            builder.move_to(first);
            
            for &point in line.path.iter().skip(1) {
                builder.line_to(world_to_screen(point, bounds, map));
            }
        });

        frame.stroke(
            &path,
            Stroke::default()
                .with_width(3.0)
                .with_color(line.color),
        );
    }

    // Draw stations
    for station in map.mrt_system.stations.values() {
        let pos = world_to_screen(station.position, bounds, map);
        let radius = if station.interchange { 4.0 } else { 3.0 };

        frame.fill(
            &Path::circle(pos, radius),
            Fill::from(Color::WHITE),
        );
    }
}

fn draw_landmarks(frame: &mut Frame, map: &SingaporeVectorMap, bounds: iced::Rectangle) {
    for landmark in &map.landmarks {
        let pos = world_to_screen(landmark.position, bounds, map);
        let (icon, color) = landmark_style(&landmark.landmark_type);
        
        frame.fill_text(iced::widget::canvas::Text {
            content: icon.to_string(),
            position: pos,
            color,
            size: 12.0.into(),
            font: iced::Font::DEFAULT,
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Center,
        });
    }
}

// Mission briefing panel
pub fn render_mission_briefing(
    briefing: &MissionBriefing,
    city: &City,
) -> Element<Message> {
    column![
        render_mission_header(city, briefing),
        render_objectives(briefing),
        render_intelligence(briefing),
        render_risks(briefing),
        button("Launch Mission").on_press(Message::Hub(HubMsg::LaunchMission))
    ]
    .spacing(10)
    .into()
}

fn render_mission_header(city: &City, briefing: &MissionBriefing) -> Element<Message> {
    let threat_color = match city.corruption_level {
        1..=3 => Color::from_rgb(0.0, 1.0, 0.0),
        4..=6 => Color::from_rgb(1.0, 1.0, 0.0),
        7..=8 => Color::from_rgb(1.0, 0.65, 0.0),
        _ => Color::from_rgb(1.0, 0.0, 0.0),
    };

    row![
        text(&city.name).size(18).color(Color::from_rgb(1.0, 0.0, 0.0)),
        text(format!("THREAT: {}", 
            match city.corruption_level {
                1..=3 => "LOW",
                4..=6 => "MODERATE",
                7..=8 => "HIGH",
                _ => "EXTREME",
            }
        )).color(threat_color),
    ]
    .spacing(20)
    .into()
}

fn render_objectives(briefing: &MissionBriefing) -> Element<Message> {
    let mut objectives = column![text("OBJECTIVES").size(16).color(Color::from_rgb(1.0, 0.0, 0.0))].spacing(5);
    
    for obj in &briefing.objectives {
        let prefix_color = if obj.required {
            Color::from_rgb(1.0, 0.0, 0.0)
        } else {
            Color::from_rgb(0.0, 0.0, 1.0)
        };
        
        objectives = objectives.push(
            column![
                row![
                    text(if obj.required { "[REQUIRED]" } else { "[OPTIONAL]" })
                        .color(prefix_color),
                    text(&obj.name),
                    text("★".repeat(obj.difficulty as usize))
                        .color(Color::from_rgb(1.0, 1.0, 0.0)),
                ].spacing(5),
                text(&obj.description).size(11).color(Color::from_rgba(0.7, 0.7, 0.7, 1.0)),
            ]
            .spacing(2)
        );
    }
    
    container(objectives)
        .padding(10)
        .style(|_| container::Appearance {
            background: Some(iced::Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.8))),
            ..Default::default()
        })
        .into()
}

fn render_intelligence(briefing: &MissionBriefing) -> Element<Message> {
    container(
        column![
            text("INTELLIGENCE").size(16).color(Color::from_rgb(0.0, 0.0, 1.0)),
            row![
                text(format!("Enemies: {}", briefing.resistance.enemy_count)),
                text(format!("Security: {}/5", briefing.resistance.security_level)),
                text(format!("Alert: {:.0}%", briefing.resistance.alert_sensitivity * 100.0)),
            ].spacing(10),
        ]
        .spacing(5)
    )
    .padding(10)
    .style(|_| container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.8))),
        ..Default::default()
    })
    .into()
}

fn render_risks(briefing: &MissionBriefing) -> Element<Message> {
    container(
        column![
            text("RISKS").size(16).color(Color::from_rgb(1.0, 1.0, 0.0)),
            text(format!("Failure Chance: {:.0}%", briefing.risks.mission_failure_chance * 100.0))
                .color(if briefing.risks.mission_failure_chance > 0.5 {
                    Color::from_rgb(1.0, 0.0, 0.0)
                } else {
                    Color::WHITE
                }),
        ]
        .spacing(5)
    )
    .padding(10)
    .style(|_| container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.8))),
        ..Default::default()
    })
    .into()
}

// Helper functions
fn world_to_screen(world: Vec2, bounds: iced::Rectangle, map: &SingaporeVectorMap) -> Point {
    let normalized = Vec2::new(
        (world.x - map.bounds.min.x) / (map.bounds.max.x - map.bounds.min.x),
        1.0 - (world.y - map.bounds.min.y) / (map.bounds.max.y - map.bounds.min.y),
    );
    
    Point::new(
        bounds.x + normalized.x * bounds.width,
        bounds.y + normalized.y * bounds.height,
    )
}

fn screen_to_world(screen: Point, bounds: iced::Rectangle, map: &SingaporeVectorMap) -> Vec2 {
    let normalized = Vec2::new(
        (screen.x - bounds.x) / bounds.width,
        (screen.y - bounds.y) / bounds.height,
    );
    
    Vec2::new(
        map.bounds.min.x + normalized.x * (map.bounds.max.x - map.bounds.min.x),
        map.bounds.max.y - normalized.y * (map.bounds.max.y - map.bounds.min.y),
    )
}

fn point_in_polygon(point: Vec2, polygon: &[Vec2]) -> bool {
    let mut inside = false;
    let mut j = polygon.len() - 1;

    for i in 0..polygon.len() {
        let pi = polygon[i];
        let pj = polygon[j];

        if ((pi.y > point.y) != (pj.y > point.y)) &&
           (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

fn brighten_iced_color(color: Color, factor: f32) -> Color {
    Color {
        r: (color.r * factor).min(1.0),
        g: (color.g * factor).min(1.0),
        b: (color.b * factor).min(1.0),
        a: color.a,
    }
}

fn landmark_style(landmark_type: &LandmarkType) -> (&'static str, Color) {
    match landmark_type {
        LandmarkType::Airport => ("✈", Color::from_rgb(0.68, 0.85, 0.9)),
        LandmarkType::Port => ("⚓", Color::from_rgb(0.75, 0.75, 0.75)),
        LandmarkType::Government => ("🏛", Color::from_rgb(1.0, 1.0, 0.0)),
        LandmarkType::Corporate => ("🏢", Color::from_rgb(1.0, 0.0, 0.0)),
        LandmarkType::University => ("🎓", Color::from_rgb(0.0, 1.0, 0.0)),
        LandmarkType::Hospital => ("🏥", Color::from_rgb(1.0, 0.4, 0.4)),
        LandmarkType::Shopping => ("🛍", Color::from_rgb(0.68, 1.0, 0.68)),
        LandmarkType::Tourist => ("📸", Color::from_rgb(1.0, 1.0, 0.68)),
    }
}
