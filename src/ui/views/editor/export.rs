use crate::models::Character;
use crate::ui::types::CharacterTab;
use crate::ui::CrapApp;
use eframe::egui;

/// Renders the EXPORT menu button - pure UI, delegates to controller
pub fn render_export_menu(ui: &mut egui::Ui, app: &CrapApp, character: &Character) {
    let on_st_tab = app.active_char_tab == CharacterTab::SillyTavern;

    ui.menu_button("EXPORT", |ui| {
        if on_st_tab {
            // On SillyTavern tab: only ST export is shown
            if ui.button("🎭 SillyTavern Card V3 (.json)").clicked() {
                app.export_character_sillytavern(character);
                ui.close_menu();
            }

            ui.separator();
            ui.label(
                egui::RichText::new("Switch to Main Data tab for\nother export formats.")
                    .size(10.0)
                    .color(egui::Color32::GRAY),
            );
        } else {
            // On any Main Data tab: standard exports
            if ui.button("Native (.crapp)").clicked() {
                app.export_character_native(character);
                ui.close_menu();
            }

            if ui.button("Character Card - spicychat.ai (.json)").clicked() {
                app.export_character_v2_json(character);
                ui.close_menu();
            }

            if ui.button("Document (.md)").clicked() {
                app.export_character_markdown(character);
                ui.close_menu();
            }

            if ui.button("Character Card (.png)").clicked() {
                app.export_character_png(character);
                ui.close_menu();
            }

            ui.separator();
            ui.label(
                egui::RichText::new("Switch to 🎭 Silly Tavern tab\nto export ST format.")
                    .size(10.0)
                    .color(egui::Color32::GRAY),
            );
        }
    });
}

/// Renders the IMPORT menu button - pure UI, delegates to controller
pub fn render_import_menu(
    ui: &mut egui::Ui,
    app: &CrapApp,
    character: &Character,
    trigger_import: &mut bool,
) {
    ui.menu_button("IMPORT", |ui| {
        if ui.button("Import File (JSON, PNG, CRAPP)").clicked() {
            let target_id = if character.id != 0 {
                Some(character.id as u64)
            } else {
                None
            };
            app.import_character_from_file(target_id);
            ui.close_menu();
        }

        if ui.button("Import from Clipboard").clicked() {
            *trigger_import = true;
            ui.close_menu();
        }
    });
}
