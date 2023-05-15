use eframe::egui::{Label, RichText, TextStyle, Ui};
use egui_dock::egui::Sense;
use egui_extras::{Column, TableBuilder};

use crate::{assembly::AssemblySelection, AppState, AppView};

pub struct LocationView {
    locations: Vec<Location>,
}

impl LocationView {
    pub fn new(locations: Vec<Location>) -> Self {
        Self { locations }
    }
}

impl AppView for LocationView {
    fn title(&self) -> String {
        "Location".into()
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
                body.rows(row_height, self.locations.len(), |index, mut row| {
                    let location = &self.locations[index];
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
                            state.selection = Some(AssemblySelection { va: location.va })
                        }
                    });
                    row.col(|ui| {
                        ui.add(
                            Label::new(
                                RichText::from(match location.type_ {
                                    LocationType::TlsCallback => "TLS callback",
                                    LocationType::EntryPoint => "Entry point",
                                    LocationType::Export => "Export",
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

pub struct Location {
    pub va: u64,
    pub type_: LocationType,
    pub name: String,
}

pub enum LocationType {
    TlsCallback,
    EntryPoint,
    Export,
}
