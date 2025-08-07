// ui_iced/research.rs - Research UI components
use iced::{
    widget::{button, column, container, row, text, progress_bar, scrollable},
    Element, Length, Color,
};
use crate::ui_iced::{Message, HubMsg, ResearchMsg};
use crate::core::*;

pub fn render_research_panel(
    research_progress: &ResearchProgress,
    research_db: &ResearchDatabase,
    global_data: &GlobalData,
) -> Element<Message> {
    column![
        render_research_header(global_data, research_progress),
        row![
            render_active_queue(research_progress, research_db),
            render_available_projects(research_progress, research_db, global_data),
            render_scientists_panel(),
        ].spacing(10),
    ]
    .spacing(10)
    .into()
}

fn render_research_header(data: &GlobalData, progress: &ResearchProgress) -> Element<Message> {
    row![
        text(format!("Credits: ${}", data.credits)).color(Color::from_rgb(1.0, 1.0, 0.0)),
        text(format!("Active: {}/5", progress.active_queue.len())),
        text(format!("Completed: {}", progress.completed.len())),
    ]
    .spacing(20)
    .into()
}

fn render_active_queue(progress: &ResearchProgress, db: &ResearchDatabase) -> Element<Message> {
    let mut queue = column![text("ACTIVE RESEARCH").size(16)].spacing(5);
    
    for active in &progress.active_queue {
        if let Some(project) = db.get_project(&active.project_id) {
            queue = queue.push(render_active_project(project, active));
        }
    }
    
    container(scrollable(queue))
        .width(Length::FillPortion(1))
        .into()
}

fn render_active_project(project: &ResearchProject, active: &ActiveResearch) -> Element<Message> {
    container(
        column![
            text(&project.name).size(14),
            progress_bar(0.0..=1.0, active.progress),
            text(format!("{:.1} days left", active.time_remaining)).size(12),
        ]
        .spacing(2)
    )
    .padding(5)
    .style(|_| container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgba(0.1, 0.1, 0.2, 0.8))),
        border: iced::Border {
            color: priority_color(active.priority),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn render_available_projects(
    progress: &ResearchProgress,
    db: &ResearchDatabase,
    data: &GlobalData,
) -> Element<Message> {
    let mut projects = column![text("AVAILABLE PROJECTS").size(16)].spacing(5);
    
    for project in db.get_available_projects(progress).iter().take(5) {
        projects = projects.push(render_project_card(project, data));
    }
    
    container(scrollable(projects))
        .width(Length::FillPortion(1))
        .into()
}

fn render_project_card(project: &ResearchProject, data: &GlobalData) -> Element<Message> {
    let can_afford = data.credits >= project.cost;
    let color = category_color(project.category);
    
    container(
        column![
            row![
                text(format!("[{:?}]", project.category)).size(12).color(color),
                text(&project.name).size(14),
            ].spacing(5),
            text(&project.description).size(11),
            row![
                text(format!("${}", project.cost))
                    .color(if can_afford { Color::WHITE } else { Color::from_rgb(1.0, 0.0, 0.0) }),
                text(format!("{:.1} days", project.base_time_days)).size(12),
                if can_afford {
                    button("Start")
                        .on_press(Message::Hub(HubMsg::Research(ResearchMsg::StartProject(project.id.clone()))))
                        .into()
                } else {
                    button("Start").into()
                }
            ].spacing(10),
        ]
        .spacing(3)
    )
    .padding(5)
    .style(|_| container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.8))),
        border: iced::Border {
            color: Color::from_rgba(0.3, 0.3, 0.3, 0.8),
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .into()
}

fn render_scientists_panel() -> Element<Message> {
    column![
        text("RESEARCH TEAM").size(16),
        text("No scientists recruited").color(Color::from_rgba(0.5, 0.5, 0.5, 1.0)),
        text("Find and recruit scientists in missions").size(11),
    ]
    .width(Length::FillPortion(1))
    .spacing(5)
    .into()
}

// Territory control panel
pub fn render_territory_panel(
    territory_manager: &TerritoryManager,
    cities_db: &CitiesDatabase,
) -> Element<Message> {
    column![
        row![
            text(format!("Controlled: {}", territory_manager.controlled_districts.len()))
                .color(Color::from_rgb(1.0, 1.0, 0.0)),
            text(format!("Daily Income: ${}", territory_manager.total_daily_income))
                .color(Color::from_rgb(0.0, 1.0, 0.0)),
        ].spacing(20),
        render_controlled_territories(territory_manager, cities_db),
    ]
    .spacing(10)
    .into()
}

fn render_controlled_territories(
    manager: &TerritoryManager,
    cities_db: &CitiesDatabase,
) -> Element<Message> {
    let mut list = column![].spacing(5);
    
    for (city_id, control) in manager.controlled_districts.iter().take(5) {
        if let Some(city) = cities_db.get_city(city_id) {
            list = list.push(render_territory_card(&city, control));
        }
    }
    
    scrollable(list).height(Length::Fill).into()
}

fn render_territory_card(city: &City, control: &DistrictControl) -> Element<Message> {
    let control_color = control_level_color(control.control_level);
    
    container(
        column![
            row![
                text(&city.name).size(14),
                text(format!("{:?}", control.control_level))
                    .color(control_color)
                    .size(12),
            ],
            progress_bar(0.0..=1.0, control.control_strength),
            text(format!("Daily: ${} | Days: {}", 
                (city.population as f32 * 1000.0 * control.control_strength) as u32,
                control.days_controlled
            )).size(11),
        ]
        .spacing(3)
    )
    .padding(5)
    .style(move |_| container::Appearance {
        background: Some(iced::Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.8))),
        border: iced::Border {
            color: control_color,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    })
    .into()
}

// Helper functions
fn priority_color(priority: ResearchPriority) -> Color {
    match priority {
        ResearchPriority::Low => Color::from_rgba(0.5, 0.5, 0.5, 1.0),
        ResearchPriority::Normal => Color::WHITE,
        ResearchPriority::High => Color::from_rgb(1.0, 1.0, 0.0),
        ResearchPriority::Critical => Color::from_rgb(1.0, 0.0, 0.0),
    }
}

fn category_color(category: ResearchCategory) -> Color {
    match category {
        ResearchCategory::Weapons => Color::from_rgb(0.8, 0.3, 0.3),
        ResearchCategory::Cybernetics => Color::from_rgb(0.3, 0.3, 0.8),
        ResearchCategory::Equipment => Color::from_rgb(0.3, 0.8, 0.3),
        ResearchCategory::Intelligence => Color::from_rgb(0.8, 0.8, 0.3),
    }
}

fn control_level_color(level: ControlLevel) -> Color {
    match level {
        ControlLevel::Corporate => Color::from_rgb(0.5, 0.5, 0.5),
        ControlLevel::Contested => Color::from_rgb(1.0, 0.65, 0.0),
        ControlLevel::Liberated => Color::from_rgb(0.4, 0.8, 0.4),
        ControlLevel::Secured => Color::from_rgb(0.2, 0.6, 0.2),
        ControlLevel::Autonomous => Color::from_rgb(0.0, 1.0, 0.4),
    }
}
