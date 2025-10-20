use crate::{
    editor_state::EditorState,
    interface::{
        events::{
            PopupMenuRequestedEvent, RequestCameraEntityFrame, RequestEditorToggle,
            RequestToggleCameraSync, RequestViewportCameraOverride, SetActiveWorld,
        },
        panels::{
            bottom_panel::{BottomDockState, BottomTab}, right_panel::{SideDockState, SideTab}, BottomTabType, SideTabType
        },
        popups::PopupType,
        EditorEvents,
    },
    viewport::ViewportCameraState,
    UI_CONFIG,
};
use bevy::{ecs::{entity::Entity, system::Commands}, prelude::{Query, ResMut, With}};
use bevy_egui::egui::{self, Button, Widget};
use bevy_granite_core::{
    absolute_asset_to_rel, entities::SaveSettings, RequestDespawnBySource,
    RequestDespawnSerializableEntities, RequestLoadEvent, RequestSaveEvent, UserInput,
};
use bevy_granite_gizmos::selection::events::EntityEvents;
use native_dialog::FileDialog;
use bevy_granite_gizmos::selection::Selected;

pub fn top_bar_ui(
    side_dock: &mut ResMut<SideDockState>,
    bottom_dock: &mut ResMut<BottomDockState>,
    ui: &mut egui::Ui,
    events: &mut EditorEvents,
    user_input: &UserInput,
    editor_state: &EditorState,
    commands: &mut Commands,
    camera_options: &[(Entity, String)],
    viewport_camera_state: &ViewportCameraState,
    selection_query: &Query<Entity, With<Selected>>,
) {
    let active_camera_label = if viewport_camera_state.is_using_editor() {
        "Editor Camera".to_string()
    } else {
        camera_options
            .iter()
            .find(|(entity, _)| Some(*entity) == viewport_camera_state.active_override)
            .map(|(_, label)| label.clone())
            .unwrap_or_else(|| "Unknown Camera".to_string())
    };

    let spacing = UI_CONFIG.spacing;

    ui.vertical(|ui| {
        ui.add_space(spacing);

        // MENUs
        ui.horizontal(|ui| {
            ui.menu_button("File", |ui| {
                ui.set_min_width(150.0);

                let save_as_button = Button::new("Save as").shortcut_text("Ctrl + Shift + S").ui(ui);
                if save_as_button.clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Granite Scene", &["scene"])
                        .show_save_single_file()
                        .unwrap()
                    {
                        events
                            .save
                            .write(RequestSaveEvent(path.display().to_string()));
                    }
                    ui.close();
                }

                let save_button = Button::new("Save").shortcut_text("Ctrl + S").ui(ui);
                if save_button.clicked() {
                    let loaded = &editor_state.loaded_sources;
                    if !loaded.is_empty() {
                        for source in loaded.iter() {
                            events.save.write(RequestSaveEvent(source.to_string()));
                        }
                    }
                    ui.close();
                }

                let open_button = Button::new("Open").shortcut_text("Ctrl + O").ui(ui);
                if open_button.clicked() {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Granite Scene", &["scene"])
                        .show_open_single_file()
                        .unwrap()
                    {
                        events.load.write(RequestLoadEvent(
                            absolute_asset_to_rel(path.display().to_string()).to_string(),
                            SaveSettings::Runtime,
                            None,
                        ));
                    }
                    ui.close();
                }

                ui.separator();

                ui.menu_button("Despawn", |ui| {
                    let despawn_all_button = Button::new("Despawn All Entities").ui(ui);
                    if despawn_all_button.clicked() {
                        events.despawn_all.write(RequestDespawnSerializableEntities);
                        ui.close();
                    }

                    ui.separator();

                    ui.label(format!(
                        "Loaded Sources ({}):",
                        editor_state.loaded_sources.len()
                    ));

                    if editor_state.loaded_sources.is_empty() {
                        ui.label("  (No sources loaded)");
                    } else {
                        let sources: Vec<String> =
                            editor_state.loaded_sources.iter().cloned().collect();
                        for source in sources {
                            if ui.button(format!("{}", source)).clicked() {
                                events
                                    .despawn_by_source
                                    .write(RequestDespawnBySource(source));
                                ui.close();
                            }
                        }
                    }
                });

                ui.menu_button("Set Active Scene", |ui| {
                    ui.label(format!(
                        "Available Sources ({}):",
                        editor_state.loaded_sources.len()
                    ));

                    if editor_state.loaded_sources.is_empty() {
                        ui.label("  (No sources loaded)");
                    } else {
                        let sources: Vec<String> =
                            editor_state.loaded_sources.iter().cloned().collect();
                        for source in sources {
                            let is_current = editor_state
                                .current_file
                                .as_ref()
                                .map(|current| current == &source)
                                .unwrap_or(false);

                            let button_text = if is_current {
                                format!("[ACTIVE] {}", source)
                            } else {
                                source.clone()
                            };

                            if ui.button(button_text).clicked() {
                                events.set_active_world.write(SetActiveWorld(source));
                                ui.close();
                            }
                        }
                    }
                });

                ui.separator();

                let open_default_world_button = Button::new("Open Default World").ui(ui);
                if open_default_world_button.clicked() {
                    events.load.write(RequestLoadEvent(
                        editor_state.default_world.clone(),
                        SaveSettings::Runtime,
                        None,
                    ));
                    ui.close();
                }

                let save_default_world_button = Button::new("Save Default World").ui(ui);
                if save_default_world_button.clicked() {
                    events
                        .save
                        .write(RequestSaveEvent(editor_state.default_world.clone()));

                    ui.close();
                }
            });

            ui.add_space(spacing);

            ui.menu_button("Edit", |ui| {
                ui.set_min_width(150.0);

                let any_selected = !selection_query.is_empty();
                let any_on_clipboard = false;

                for (text, shortcut, event, enabled) in [
                    ("Cut", "Ctrl + X", EntityEvents::Cut, any_selected),
                    ("Copy", "Ctrl + C", EntityEvents::Copy, any_selected),
                    ("Paste", "Ctrl + V", EntityEvents::Paste, any_on_clipboard),
                    // @TODO disable if no active selected entity
                    ("Deselect All", "U", EntityEvents::DeselectAll, any_selected),
                ] {
                    ui.add_enabled_ui(enabled, |ui| {
                        let button = Button::new(text).shortcut_text(shortcut).ui(ui);
                        if button.clicked() {
                            commands.trigger(event);
                        }
                    });
                }
            });

            ui.add_space(spacing);

            ui.menu_button("View", |ui| {
                ui.set_min_width(150.0);

                let toggle_editor_button = Button::new("Toggle Editor").shortcut_text("F2").ui(ui);
                if toggle_editor_button.clicked() {
                    events.toggle_editor.write(RequestEditorToggle);
                }

                let toggle_camera_control_button = Button::new("Toggle Camera Control").shortcut_text("F3").ui(ui);
                if toggle_camera_control_button.clicked() {
                    events.toggle_cam_sync.write(RequestToggleCameraSync);
                }

                ui.separator();

                ui.menu_button("Viewport Camera", |ui| {
                    let using_editor = viewport_camera_state.is_using_editor();
                    if ui
                        .selectable_label(using_editor, "Editor Camera")
                        .clicked()
                        && !using_editor
                    {
                        events
                            .viewport_camera
                            .write(RequestViewportCameraOverride { camera: None });
                        ui.close();
                    }

                    if camera_options.is_empty() {
                        ui.label("No scene cameras targeting the primary window");
                    } else {
                        for (entity, label) in camera_options.iter() {
                            let is_active =
                                viewport_camera_state.active_override == Some(*entity);
                            if ui.selectable_label(is_active, label).clicked() && !is_active {
                                events.viewport_camera.write(RequestViewportCameraOverride {
                                    camera: Some(*entity),
                                });
                                ui.close();
                            }
                        }
                    }
                });

                ui.separator();

                // @TODO disable if no active selected entity
                let any_selected = !selection_query.is_empty();
                ui.add_enabled_ui(any_selected, |ui| {
                    let frame_active_button = Button::new("Frame Active").shortcut_text("F").ui(ui);
                    if frame_active_button.clicked() {
                            events.frame.write(RequestCameraEntityFrame);
                        }
                    });

                ui.separator();

                // @TODO Merge Side tab and Bottom tab into one enum and use a single loop to render them.

                for (tab_type, label) in vec![
                    (SideTabType::EntityEditor, "Entity Editor"),
                    (SideTabType::NodeTree, "Entities"),
                    (SideTabType::EditorSettings, "Editor Settings"),
                ] {
                    let tab = side_dock.dock_state.find_tab_from(|tab| tab.get_type() == tab_type);
                    let mut show = tab.is_some();
                    let checkbox = ui.checkbox(&mut show, label);
                    if checkbox.clicked() {
                        match tab {
                            Some(tab) => {
                                side_dock.dock_state.remove_tab(tab);
                            }
                            None => {
                                let tab = SideTab::default_from_type(tab_type);
                                side_dock.dock_state.push_to_focused_leaf(tab);
                            }
                        }
                    }
                }

                ui.separator();

                for (tab_type, label) in [
                    (BottomTabType::Log, "Log"),
                    (BottomTabType::Debug, "Debug"),
                    (BottomTabType::Events, "Events"),
                ] {
                    let tab = bottom_dock.dock_state.find_tab_from(|tab| tab.get_type() == tab_type);
                    let mut show = tab.is_some();
                    let checkbox = ui.checkbox(&mut show, label);
                    if checkbox.clicked() {
                        match tab {
                            Some(tab) => {
                                bottom_dock.dock_state.remove_tab(tab);
                            }
                            None => {
                                let tab = BottomTab::default_from_type(tab_type);
                                bottom_dock.dock_state.push_to_focused_leaf(tab);
                            }
                        }
                    }
                }
            });

            ui.add_space(spacing);

            ui.menu_button("Help", |ui| {
                ui.set_min_width(150.0);

                let show_help_button = Button::new("Show Help").shortcut_text("F1").ui(ui);
                if show_help_button.clicked() {
                    events.popup.write(PopupMenuRequestedEvent {
                        popup: PopupType::Help,
                        mouse_pos: user_input.mouse_pos,
                    });
                }
            });
        });

        ui.separator();

        // Buttons
        ui.horizontal(|ui| {
            ui.separator();
            let add_entity_button = Button::new("Add Entity").shortcut_text("Shift + A").ui(ui);
            if add_entity_button.clicked() {
                events.popup.write(PopupMenuRequestedEvent {
                    popup: PopupType::AddEntity,
                    mouse_pos: user_input.mouse_pos,
                });
            }
            ui.separator();
            let parents_button = Button::new("Parents").shortcut_text("Shift + P").ui(ui);
            if parents_button.clicked() {
                events.popup.write(PopupMenuRequestedEvent {
                    popup: PopupType::AddRelationship,
                    mouse_pos: user_input.mouse_pos,
                });
            }

            ui.separator();
            ui.label(format!("Viewing: {}", active_camera_label));
        });

        ui.add_space(spacing);
    });
}
