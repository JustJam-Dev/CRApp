use crate::models::{count_tokens, Character};
use crate::ui::types::StTab;
use crate::ui::CrapApp;
use eframe::egui;
use egui_cosmic_text::cosmic_text::Family;
use crate::ui::types::EditorFontFamily;

pub fn render_sillytavern_tab(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    character: &mut Character,
    status_update: &mut Option<(String, egui::Color32)>,
) {
    let font_family = match app.editor_font {
        EditorFontFamily::SansSerif => Family::SansSerif,
        EditorFontFamily::Serif => Family::Serif,
        EditorFontFamily::Monospace => Family::Monospace,
    };

    // Info banner
    egui::Frame::none()
        .fill(egui::Color32::from_rgb(30, 60, 30))
        .rounding(6.0)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("🎭 SillyTavern Format")
                        .strong()
                        .color(egui::Color32::from_rgb(100, 220, 100)),
                );
                ui.label(
                    egui::RichText::new(
                        "  These fields are independent from Main Data and export as a V3 character card.",
                    )
                    .size(11.0)
                    .color(egui::Color32::from_rgb(160, 200, 160)),
                );
            });
        });

    ui.add_space(6.0);

    // Sub-tabs: Main | Advanced
    ui.horizontal(|ui| {
        ui.selectable_value(&mut app.active_st_tab, StTab::Main, "Main");
        ui.selectable_value(&mut app.active_st_tab, StTab::Advanced, "Advanced");
    });
    ui.separator();

    match app.active_st_tab {
        StTab::Main => render_st_main(app, ui, character, status_update, font_family),
        StTab::Advanced => render_st_advanced(app, ui, character, status_update, font_family),
    }
}

// ── Helper macro to reduce repetition for text fields ──────────────────────

fn st_field(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    label: &str,
    field_key: &str,
    value: &mut String,
    desired_lines: usize,
    status_update: &mut Option<(String, egui::Color32)>,
    font_family: Family,
) {
    let id = ui.make_persistent_id(format!("st_header_{}", field_key));
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
        .show_header(ui, |ui| {
            ui.label(label);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Copy").clicked() {
                    ui.output_mut(|o| o.copied_text = value.clone());
                    *status_update = Some((
                        format!("Copied {} to clipboard", label),
                        egui::Color32::GREEN,
                    ));
                }
                ui.label(
                    egui::RichText::new(format!(
                        "Tokens: {} | Chars: {}",
                        count_tokens(value),
                        value.chars().count()
                    ))
                    .size(12.0)
                    .color(egui::Color32::GRAY),
                );
            });
        })
        .body(|ui| {
            crate::ui::components::CodeEditor::new(value, &format!("st_{}_editor", field_key), font_family)
                .desired_lines(desired_lines)
                .font_size_offset(if app.editor_large_font { 2.0 } else { 0.0 })
                .bright_mode(app.editor_bright_mode)
                .highlight(app.editor_search_query.clone())
                .spell_check(
                    if app.enable_spell_check {
                        app.spell_checker.clone()
                    } else {
                        None
                    },
                )
                .show(
                    ui,
                    &mut app.cosmic_font_system,
                    &mut app.cosmic_swash_cache,
                    &mut app.cosmic_atlas,
                    &mut app.cosmic_editors,
                    &mut app.cosmic_clipboard,
                );
        });
    ui.add_space(6.0);
}

// ── Main sub-tab ────────────────────────────────────────────────────────────

fn render_st_main(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    character: &mut Character,
    status_update: &mut Option<(String, egui::Color32)>,
    font_family: Family,
) {
    ui.horizontal(|ui| {
        let available_width = ui.available_width();
        let left_width = available_width * 0.72;

        ui.allocate_ui_with_layout(
            egui::vec2(left_width, ui.available_height()),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                // Name (single line)
                ui.label("Name");
                crate::ui::components::CodeEditor::new(
                    &mut character.st_name,
                    "st_name_editor",
                    font_family,
                )
                .single_line()
                .font_size_offset(if app.editor_large_font { 2.0 } else { 0.0 })
                .bright_mode(app.editor_bright_mode)
                .highlight(app.editor_search_query.clone())
                .spell_check(None)
                .show(
                    ui,
                    &mut app.cosmic_font_system,
                    &mut app.cosmic_swash_cache,
                    &mut app.cosmic_atlas,
                    &mut app.cosmic_editors,
                    &mut app.cosmic_clipboard,
                );
                ui.add_space(6.0);

                // Description (ST equivalent of "personality" in main data)
                st_field(app, ui, "Description", "description", &mut character.st_description, 10, status_update, font_family);

                // Personality (ST's short personality summary)
                st_field(app, ui, "Personality Summary", "personality", &mut character.st_personality, 4, status_update, font_family);

                // Scenario
                st_field(app, ui, "Scenario", "scenario", &mut character.st_scenario, 8, status_update, font_family);

                // First Message
                st_field(app, ui, "First Message (Greeting)", "first_mes", &mut character.st_first_mes, 10, status_update, font_family);

                // Example Dialogue
                st_field(app, ui, "Example Dialogue", "mes_example", &mut character.st_mes_example, 8, status_update, font_family);

                // Creator Notes
                st_field(app, ui, "Creator's Notes", "creator_notes", &mut character.st_creator_notes, 4, status_update, font_family);
            },
        );

        // Right panel: token summary
        ui.vertical(|ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_gray(30))
                .rounding(6.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Token Summary")
                            .strong()
                            .color(egui::Color32::WHITE),
                    );
                    ui.separator();

                    let fields = [
                        ("Description", &character.st_description),
                        ("Personality", &character.st_personality),
                        ("Scenario", &character.st_scenario),
                        ("First Message", &character.st_first_mes),
                        ("Example Dialogue", &character.st_mes_example),
                        ("Creator Notes", &character.st_creator_notes),
                    ];

                    let mut total_tokens = 0;
                    for (label, text) in &fields {
                        let t = count_tokens(text);
                        total_tokens += t;
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{}:", label))
                                    .size(11.0)
                                    .color(egui::Color32::GRAY),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(format!("{}", t))
                                            .size(11.0)
                                            .color(egui::Color32::WHITE),
                                    );
                                },
                            );
                        });
                    }

                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Total:").strong().color(egui::Color32::WHITE));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(format!("{}", total_tokens))
                                    .strong()
                                    .color(egui::Color32::from_rgb(100, 220, 100)),
                            );
                        });
                    });
                });
        });
    });
}

// ── Advanced sub-tab ────────────────────────────────────────────────────────

fn render_st_advanced(
    app: &mut CrapApp,
    ui: &mut egui::Ui,
    character: &mut Character,
    status_update: &mut Option<(String, egui::Color32)>,
    font_family: Family,
) {
    // System Prompt
    st_field(app, ui, "System Prompt", "system_prompt", &mut character.st_system_prompt, 6, status_update, font_family);

    // Post History Instructions
    st_field(app, ui, "Post History Instructions", "post_history_instructions", &mut character.st_post_history_instructions, 6, status_update, font_family);

    ui.separator();

    // Alternate Greetings
    egui::CollapsingHeader::new("Alternate Greetings")
        .default_open(true)
        .show(ui, |ui| {
            let count = character.st_alternate_greetings.len();
            ui.label(
                egui::RichText::new(format!("{} alternate greeting(s)", count))
                    .size(11.0)
                    .color(egui::Color32::GRAY),
            );

            let mut to_remove: Option<usize> = None;
            for (idx, greeting) in character.st_alternate_greetings.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("#{}", idx + 1))
                            .size(11.0)
                            .color(egui::Color32::from_rgb(100, 180, 255)),
                    );
                    if ui.small_button("✖ Remove").clicked() {
                        to_remove = Some(idx);
                    }
                    ui.label(
                        egui::RichText::new(format!(
                            "Tokens: {} | Chars: {}",
                            count_tokens(greeting),
                            greeting.chars().count()
                        ))
                        .size(10.0)
                        .color(egui::Color32::GRAY),
                    );
                });

                crate::ui::components::CodeEditor::new(
                    greeting,
                    &format!("st_alt_greeting_{}_editor", idx),
                    font_family,
                )
                .desired_lines(6)
                .font_size_offset(if app.editor_large_font { 2.0 } else { 0.0 })
                .bright_mode(app.editor_bright_mode)
                .highlight(app.editor_search_query.clone())
                .spell_check(if app.enable_spell_check { app.spell_checker.clone() } else { None })
                .show(
                    ui,
                    &mut app.cosmic_font_system,
                    &mut app.cosmic_swash_cache,
                    &mut app.cosmic_atlas,
                    &mut app.cosmic_editors,
                    &mut app.cosmic_clipboard,
                );
                ui.add_space(4.0);
            }

            if let Some(idx) = to_remove {
                character.st_alternate_greetings.remove(idx);
            }

            if ui.button("+ Add Alternate Greeting").clicked() {
                character.st_alternate_greetings.push(String::new());
                *status_update = Some(("Added alternate greeting".to_string(), egui::Color32::GREEN));
            }
        });

    ui.add_space(8.0);
    ui.separator();

    // Metadata row
    egui::CollapsingHeader::new("Metadata")
        .default_open(true)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Creator:");
                ui.add(
                    egui::TextEdit::singleline(&mut character.st_creator)
                        .desired_width(200.0)
                        .hint_text("Author name..."),
                );

                ui.add_space(16.0);

                ui.label("Character Version:");
                ui.add(
                    egui::TextEdit::singleline(&mut character.st_character_version)
                        .desired_width(120.0)
                        .hint_text("e.g. 1.0"),
                );
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Talkativeness:");
                let mut talk_f32 = character.st_talkativeness as f32;
                if ui
                    .add(
                        egui::Slider::new(&mut talk_f32, 0.0..=1.0)
                            .step_by(0.05)
                            .fixed_decimals(2),
                    )
                    .changed()
                {
                    character.st_talkativeness = talk_f32 as f64;
                }
                ui.label(
                    egui::RichText::new("(0 = quiet, 1 = very talkative)")
                        .size(11.0)
                        .color(egui::Color32::GRAY),
                );
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("World / Lorebook Name:");
                ui.add(
                    egui::TextEdit::singleline(&mut character.st_world)
                        .desired_width(250.0)
                        .hint_text("Linked world name..."),
                );
            });
        });

    ui.add_space(8.0);
    ui.separator();

    // Depth Prompt
    egui::CollapsingHeader::new("Depth Prompt (Character Note)")
        .default_open(true)
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(
                    "Injected into context at the specified depth. Useful for reinforcing character behavior.",
                )
                .size(11.0)
                .color(egui::Color32::GRAY),
            );
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Depth:");
                let mut depth_i32 = character.st_depth_prompt_depth as i32;
                if ui
                    .add(egui::DragValue::new(&mut depth_i32).clamp_range(0..=100))
                    .changed()
                {
                    character.st_depth_prompt_depth = depth_i32 as i64;
                }

                ui.add_space(16.0);

                ui.label("Role:");
                egui::ComboBox::from_id_source("st_depth_role_combo")
                    .selected_text(&character.st_depth_prompt_role)
                    .show_ui(ui, |ui| {
                        for role in &["system", "user", "assistant"] {
                            ui.selectable_value(
                                &mut character.st_depth_prompt_role,
                                role.to_string(),
                                *role,
                            );
                        }
                    });
            });

            ui.add_space(4.0);

            st_field(
                app,
                ui,
                "Depth Prompt Text",
                "depth_prompt",
                &mut character.st_depth_prompt,
                6,
                status_update,
                font_family,
            );
        });
}
