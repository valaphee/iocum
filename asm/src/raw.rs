use eframe::egui::{Label, RichText, TextStyle, Ui};
use egui_extras::{Column, TableBuilder};
use iced_x86::OpCodeOperandKind::al;

use crate::{AppState, AppView};

pub struct RawView {
    va: u64,
    data: Vec<u8>,
}

impl RawView {
    pub fn new(va: u64, data: Vec<u8>) -> Self {
        Self { va, data }
    }
}

impl AppView for RawView {
    fn title(&self) -> String {
        format!("Raw ({:016X})", self.va)
    }

    fn ui(&mut self, _state: &mut AppState, ui: &mut Ui) {
        let row_height = ui.text_style_height(&TextStyle::Monospace);
        TableBuilder::new(ui)
            .min_scrolled_height(0.0)
            .max_scroll_height(f32::INFINITY)
            .columns(Column::auto(), 2)
            .column(Column::remainder())
            .header(row_height, |mut row| {
                row.col(|_ui| {});
                row.col(|ui| {
                    ui.monospace("00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F");
                });
                row.col(|_ui| {});
            })
            .body(|body| {
                let align = self.va as usize / 16;
                let align_offset = self.va as usize % 16;
                body.rows(
                    row_height,
                    (self.data.len() + align_offset).div_ceil(16),
                    |index, mut row| {
                        let data = &self.data[if index == 0 {
                            0
                        } else {
                            index * 16 - align_offset
                        }
                            ..(index * 16 + 16 - align_offset).min(self.data.len())];
                        row.col(|ui| {
                            ui.add(
                                Label::new(
                                    RichText::from(format!("{:016X}", (index + align) * 16))
                                        .monospace(),
                                )
                                .wrap(false),
                            );
                        });
                        row.col(|ui| {
                            let mut text = data
                                .iter()
                                .map(|&elem| format!("{:02X}", elem))
                                .collect::<Vec<_>>()
                                .join(" ");
                            if index == 0 {
                                text = format!("{}{}", "   ".repeat(align_offset as usize), text);
                            }
                            ui.add(Label::new(RichText::from(text).monospace()).wrap(false));
                        });
                        row.col(|ui| {
                            let mut text = data
                                .iter()
                                .map(|&elem| {
                                    if elem >= 0x20 && elem <= 0x7F {
                                        elem as char
                                    } else {
                                        '.'
                                    }
                                })
                                .collect::<String>();
                            if index == 0 {
                                text = format!("{}{}", " ".repeat(align_offset as usize), text);
                            }
                            ui.add(Label::new(RichText::from(text).monospace()).wrap(false));
                        });
                    },
                );
            });
    }
}
