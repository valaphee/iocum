use eframe::egui::{Label, RichText, TextStyle, Ui};
use egui_dock::egui::Sense;
use egui_extras::{Column, TableBuilder};

use crate::{AppState, AppView};

pub struct EntryView {
    entries: Vec<Entry>,
}

impl EntryView {
    pub fn new(entries: Vec<Entry>) -> Self {
        Self { entries }
    }
}

impl AppView for EntryView {
    fn title(&self) -> String {
        "Entries".into()
    }

    fn ui(&mut self, state: &mut AppState, ui: &mut Ui) {
        let row_height = ui.text_style_height(&TextStyle::Monospace);
        TableBuilder::new(ui)
            .striped(true)
            .min_scrolled_height(0.0)
            .max_scroll_height(f32::INFINITY)
            .column(Column::auto().resizable(true))
            .column(Column::auto().resizable(true))
            .column(Column::remainder())
            .header(row_height, |mut row| {
                row.col(|ui| {
                    ui.monospace("VA");
                });
                row.col(|ui| {
                    ui.monospace("Type");
                });
                row.col(|ui| {
                    ui.monospace("Name");
                });
            })
            .body(|body| {
                body.rows(row_height, self.entries.len(), |index, mut row| {
                    let location = &self.entries[index];
                    row.col(|ui| {
                        if ui
                            .add(
                                Label::new(
                                    RichText::from(format!("{:016X}", location.va)).monospace(),
                                )
                                .wrap(false)
                                .sense(Sense::click()),
                            )
                            .clicked()
                        {
                            state.go_to_assembly_va = Some(location.va)
                        }
                    });
                    row.col(|ui| {
                        ui.add(
                            Label::new(
                                RichText::from(match location.type_ {
                                    EntryType::Main => "Main",
                                    EntryType::Export => "Export",
                                    EntryType::TlsCallback => "TLS callback",
                                })
                                .monospace(),
                            )
                            .wrap(false),
                        );
                    });
                    row.col(|ui| {
                        ui.add(Label::new(RichText::from(&location.name).monospace()).wrap(false));
                    });
                });
            });
    }
}

pub struct Entry {
    va: u64,
    type_: EntryType,
    name: String,
}

impl Entry {
    pub fn new(va: u64, type_: EntryType, name: String) -> Self {
        Self { va, type_, name }
    }
}

pub enum EntryType {
    Main,
    Export,
    TlsCallback,
}
