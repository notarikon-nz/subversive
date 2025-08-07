// ui_iced/inventory.rs - Inventory grid system
use iced::{
    widget::{button, column, container, row, text, Button, Container, Row, Column},
    Element, Length, Color, Background, Border,
};
use crate::ui_iced::{Message, MissionMsg};
use crate::core::*;
use crate::systems::ui::enhanced_inventory::*;

pub fn render_inventory_grid(
    grid: &InventoryGrid,
    selected: Option<(usize, usize)>,
) -> Element<Message> {
    let slots = (0..grid.height).map(|y| {
        row((0..grid.width).map(|x| {
            render_slot(grid, (x, y), selected)
        }))
        .spacing(2)
        .into()
    });

    Column::with_children(slots)
        .spacing(2)
        .into()
}

fn render_slot(
    grid: &InventoryGrid,
    pos: (usize, usize),
    selected: Option<(usize, usize)>,
) -> Element<Message> {
    let (x, y) = pos;
    let slot = &grid.slots[y][x];
    let is_selected = selected == Some(pos);
    
    let content = if let Some(item_slot) = slot {
        let rarity_color = rarity_to_color(item_slot.item.rarity);
        let icon = item_type_icon(&item_slot.item.item_type);
        
        container(
            column![
                text(icon).size(20),
                if item_slot.item.quantity > 1 {
                    text(format!("{}", item_slot.item.quantity)).size(10)
                } else {
                    text("").size(10)
                }
            ]
        )
        .style(move |_| container::Appearance {
            background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.8))),
            border: Border {
                color: if is_selected { Color::from_rgb(1.0, 1.0, 0.0) } else { rarity_color },
                width: 2.0,
                radius: 2.0.into(),
            },
            ..Default::default()
        })
    } else {
        container(text(""))
            .style(move |_| container::Appearance {
                background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.1, 0.8))),
                border: Border {
                    color: if is_selected { Color::from_rgb(1.0, 1.0, 0.0) } else { Color::from_rgba(0.3, 0.3, 0.3, 0.5) },
                    width: 1.0,
                    radius: 2.0.into(),
                },
                ..Default::default()
            })
    };

    button(content)
        .width(48)
        .height(48)
        .on_press(Message::Mission(MissionMsg::SelectSlot(x, y)))
        .into()
}

fn rarity_to_color(rarity: ItemRarity) -> Color {
    match rarity {
        ItemRarity::Common => Color::WHITE,
        ItemRarity::Uncommon => Color::from_rgb(0.0, 1.0, 0.0),
        ItemRarity::Rare => Color::from_rgb(0.0, 0.0, 1.0),
        ItemRarity::Epic => Color::from_rgb(0.5, 0.0, 0.5),
        ItemRarity::Legendary => Color::from_rgb(1.0, 0.65, 0.0),
        ItemRarity::Unique => Color::from_rgb(1.0, 0.0, 0.0),
    }
}

fn item_type_icon(item_type: &ItemType) -> &'static str {
    match item_type {
        ItemType::Weapon(_) => "🔫",
        ItemType::Tool(_) => "🔧",
        ItemType::Cybernetic(_) => "🧠",
        ItemType::Attachment(_) => "🔩",
        ItemType::Consumable => "💊",
        ItemType::Material => "📦",
        ItemType::Intel => "📄",
        ItemType::Currency => "💰",
    }
}

pub fn render_inventory_tabs(current_tab: InventoryTab) -> Element<Message> {
    row![
        tab_btn("ALL", InventoryTab::All, current_tab),
        tab_btn("WEAPONS", InventoryTab::Weapons, current_tab),
        tab_btn("GEAR", InventoryTab::Gear, current_tab),
        tab_btn("CONSUMABLES", InventoryTab::Consumables, current_tab),
        tab_btn("MATERIALS", InventoryTab::Materials, current_tab),
        tab_btn("INTEL", InventoryTab::Intel, current_tab),
    ]
    .spacing(5)
    .into()
}

fn tab_btn(label: &str, tab: InventoryTab, current: InventoryTab) -> Button<Message> {
    let btn = button(text(label).size(14));
    if tab == current {
        btn.style(iced::theme::Button::Primary)
    } else {
        btn
    }
    // Would connect to inventory tab change message
}

pub fn render_item_details(item: &InventoryItem) -> Element<Message> {
    column![
        text(&item.name).size(18),
        text(format!("{:?}", item.rarity)).color(rarity_to_color(item.rarity)),
        text(&item.description).size(12),
        text(format!("Value: ${} | Weight: {:.1}kg", item.value, item.weight)).size(12),
        render_item_stats(&item.stats),
    ]
    .spacing(5)
    .into()
}

fn render_item_stats(stats: &ItemStats) -> Element<Message> {
    let mut stat_list = column![].spacing(2);
    
    if stats.damage != 0 {
        stat_list = stat_list.push(stat_row("Damage", stats.damage));
    }
    if stats.accuracy != 0 {
        stat_list = stat_list.push(stat_row("Accuracy", stats.accuracy));
    }
    if stats.range != 0 {
        stat_list = stat_list.push(stat_row("Range", stats.range));
    }
    if stats.armor != 0 {
        stat_list = stat_list.push(stat_row("Armor", stats.armor));
    }
    if stats.stealth != 0 {
        stat_list = stat_list.push(stat_row("Stealth", stats.stealth));
    }
    if stats.hacking != 0 {
        stat_list = stat_list.push(stat_row("Hacking", stats.hacking));
    }
    
    stat_list.into()
}

fn stat_row(label: &str, value: i16) -> Element<Message> {
    let color = if value > 0 {
        Color::from_rgb(0.0, 1.0, 0.0)
    } else {
        Color::from_rgb(1.0, 0.0, 0.0)
    };
    
    row![
        text(label).size(12),
        text(format!("{:+}", value)).size(12).color(color)
    ]
    .spacing(10)
    .into()
}