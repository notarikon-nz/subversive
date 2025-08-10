/*
=== BENEFITS OF THIS SYSTEM ===

1. **Dramatically Less Code**: 50 lines instead of 140+
2. **External Layout Files**: UI designers can work without touching Rust
3. **Runtime Variables**: Dynamic content without recompilation  
4. **WYSIWYG Ready**: JSON structure perfect for visual editors
5. **Consistent Styling**: Constants ensure visual consistency
6. **Easy Iteration**: Change layouts without recompiling
7. **Version Control Friendly**: Separate UI changes from logic changes

=== WYSIWYG EDITOR READY ===

The JSON structure directly maps to visual properties:
- Drag/drop creates element hierarchy
- Property panels edit style values
- Live preview shows real layout
- Export generates working JSON
- Variables allow runtime customization

=== EXTENDING THE SYSTEM ===

Add new element types:
```rust
// 1. Add to enum
#[serde(rename_all = "lowercase")]
pub enum ElementType {
    Container, Text, Button, Panel, Spacer,
    Image,        // New
    Slider,       // New  
    Checkbox,     // New
}

// 2. Add to spawn_element match
ElementType::Image => {
    if let Some(src) = &element.text { // Use text field for image path
        entity_commands.insert(ImageBundle {
            image: asset_server.load(src).into(),
            ..default()
        });
    }
},
```

Add new style properties:
```rust
// 1. Add to ElementStyle
pub struct ElementStyle {
    // existing fields...
    pub opacity: Option<f32>,
    pub transform: Option<String>,
}

// 2. Add to create_base_bundle
if let Some(opacity) = style.opacity {
    entity_commands.insert(Visibility::from_alpha(opacity));
}
```

=== PERFORMANCE ===

- Layout parsing: One-time cost at startup
- Runtime spawning: Same as manual code
- Memory usage: Minimal - just JSON cache
- Variables: HashMap lookup, very fast
- File watching: Can add hot-reload for development

=== FILE STRUCTURE ===
```
project_root/
├── ui_layouts/
│   ├── main_menu.json
│   ├── settings.json  
│   ├── credits.json
│   ├── hub.json
│   └── common/
│       ├── buttons.json
│       └── panels.json
├── src/
│   └── systems/ui/layout/
│       ├── mod.rs       (main system)
│       ├── constants.rs (helpers & constants)
│       └── parser.rs    (future: advanced parsing)
```

This system will save you hundreds of lines of UI code and make your game much more maintainable!
*/