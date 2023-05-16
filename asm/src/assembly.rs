use eframe::egui::{Align, Color32, Label, Layout, RichText, Sense, TextStyle, Ui, Vec2};
use egui_extras::{Column, TableBuilder};
use iced_x86::{Formatter, FormatterTextKind, Instruction, NasmFormatter};

use crate::{AppState, AppView};

pub struct AssemblyView {
    formatter: NasmFormatter,

    instructions: Vec<Instruction>,
}

impl AssemblyView {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self {
            formatter: Default::default(),
            instructions,
        }
    }
}

impl AppView for AssemblyView {
    fn title(&self) -> String {
        format!("Assembly ({:016X})", self.instructions[0].ip()).into()
    }

    fn ui(&mut self, state: &mut AppState, ui: &mut Ui) {
        let row_height = ui.text_style_height(&TextStyle::Monospace);
        TableBuilder::new(ui)
            .min_scrolled_height(0.0)
            .max_scroll_height(f32::INFINITY)
            .column(Column::auto().resizable(true))
            .column(Column::remainder())
            .body(|body| {
                body.rows(row_height, self.instructions.len(), |index, mut row| {
                    let instruction = &self.instructions[index];
                    row.col(|ui| {
                        ui.add(
                            Label::new(
                                RichText::from(format!("{:016X}", instruction.ip())).monospace(),
                            )
                            .wrap(false),
                        );
                    });
                    row.col(|ui| {
                        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                            let mut output = FormatterOutput::new();
                            self.formatter.format(instruction, &mut output);

                            ui.spacing_mut().item_spacing = Vec2::ZERO;
                            for (text, kind) in output.0 {
                                match kind {
                                    FormatterTextKind::LabelAddress
                                    | FormatterTextKind::FunctionAddress => {
                                        let va = u64::from_str_radix(&text[..text.len() - 1], 16)
                                            .unwrap();
                                        if ui
                                            .add(
                                                Label::new(RichText::from(text).monospace())
                                                    .wrap(false)
                                                    .sense(Sense::click()),
                                            )
                                            .clicked()
                                        {
                                            state.go_to_va = Some(va)
                                        }
                                    }
                                    _ => {
                                        ui.add(
                                            Label::new(
                                                RichText::from(text)
                                                    .color(match kind {
                                                        FormatterTextKind::Mnemonic => {
                                                            Color32::LIGHT_RED
                                                        }
                                                        FormatterTextKind::Number => {
                                                            Color32::LIGHT_GREEN
                                                        }
                                                        FormatterTextKind::Register => {
                                                            Color32::LIGHT_BLUE
                                                        }
                                                        _ => Color32::WHITE,
                                                    })
                                                    .monospace(),
                                            )
                                            .wrap(false),
                                        );
                                    }
                                }
                            }
                        });
                    });
                });
            });
    }
}

struct FormatterOutput(Vec<(String, FormatterTextKind)>);

impl FormatterOutput {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl iced_x86::FormatterOutput for FormatterOutput {
    fn write(&mut self, text: &str, kind: FormatterTextKind) {
        self.0.push((String::from(text), kind));
    }
}
