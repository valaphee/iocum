use byteorder::{ReadBytesExt, LE};
use eframe::egui::{Context, Ui, WidgetText};
use egui_dock::{DockArea, Node, Tree};
use iced_x86::{Decoder, DecoderOptions};
use object::{
    coff::CoffHeader,
    pe,
    pe::{ImageDosHeader, ImageNtHeaders64, ImageTlsDirectory64},
    read::pe::{ImageNtHeaders, ImageOptionalHeader},
    LittleEndian, ReadRef,
};

use crate::{
    assembly::AssemblyView,
    entry::{Entry, EntryType, EntryView},
};

mod assembly;
mod entry;

pub fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Amalgam",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(App::new())),
    )
}

struct App {
    state: AppState,
    tree: Tree<Box<dyn AppView>>,

    data: Vec<u8>,
}

impl App {
    fn new() -> Self {
        let Some(path) = rfd::FileDialog::new().pick_file() else {
            todo!()
        };
        let Ok(data) = std::fs::read(path) else {
            todo!()
        };

        let mut self_ = Self {
            state: Default::default(),
            tree: Default::default(),
            data,
        };
        self_.open_entry_view();
        self_
    }

    fn open_view(&mut self, view: Box<dyn AppView>) {
        let title = view.title();
        if let Some((node_index, tab_index)) =
            self.tree
                .iter()
                .enumerate()
                .find_map(|(node_index, node)| match node {
                    Node::Leaf { tabs, .. } => {
                        if let Some((tab_index, _)) = tabs
                            .iter()
                            .enumerate()
                            .find(|(_, tab)| tab.title() == title)
                        {
                            Some((node_index, tab_index))
                        } else {
                            None
                        }
                    }
                    _ => None,
                })
        {
            self.tree.set_focused_node(node_index.into());
            self.tree
                .set_active_tab(node_index.into(), tab_index.into());
        } else {
            self.tree.push_to_first_leaf(view)
        }
    }

    fn open_entry_view(&mut self) {
        let data = self.data.as_slice();
        let dos_header = ImageDosHeader::parse(data).unwrap();
        let mut nt_header_offset = dos_header.nt_headers_offset().into();
        let (nt_headers, data_directories) =
            ImageNtHeaders64::parse(data, &mut nt_header_offset).unwrap();
        let file_header = nt_headers.file_header();
        let optional_header = nt_headers.optional_header();
        let sections = file_header.sections(data, nt_header_offset).unwrap();
        let mut entries = vec![];
        if optional_header.address_of_entry_point() != 0 {
            entries.push(Entry::new(
                optional_header.address_of_entry_point() as u64 + optional_header.image_base(),
                EntryType::Main,
                "".to_string(),
            ));
        }
        if let Some(directory_location) = data_directories.get(pe::IMAGE_DIRECTORY_ENTRY_TLS) {
            if let Ok(directory_data) = directory_location.data(data, &sections) {
                if let Ok(directory) = directory_data.read_at::<ImageTlsDirectory64>(0) {
                    if let Some(mut callback_data) = sections.pe_data_at(
                        data,
                        (directory.address_of_call_backs.get(LittleEndian)
                            - optional_header.image_base()) as u32,
                    ) {
                        loop {
                            let callback = callback_data.read_u64::<LE>().unwrap();
                            if callback == 0 {
                                break;
                            }
                            entries.push(Entry::new(
                                callback,
                                EntryType::TlsCallback,
                                "".to_string(),
                            ))
                        }
                    }
                }
            }
        }
        self.open_view(Box::new(EntryView::new(entries)));
    }

    fn open_assembly_view(&mut self, va: u64) {
        let data = self.data.as_slice();
        let dos_header = ImageDosHeader::parse(data).unwrap();
        let mut nt_header_offset = dos_header.nt_headers_offset().into();
        let (nt_headers, _data_directories) =
            ImageNtHeaders64::parse(data, &mut nt_header_offset).unwrap();
        let file_header = nt_headers.file_header();
        if file_header.machine.get(LittleEndian) != pe::IMAGE_FILE_MACHINE_I386
            && file_header.machine.get(LittleEndian) != pe::IMAGE_FILE_MACHINE_AMD64
        {
            return;
        }
        let optional_header = nt_headers.optional_header();
        let sections = file_header.sections(data, nt_header_offset).unwrap();
        if let Some(instruction_data) =
            sections.pe_data_at(data, (va - optional_header.image_base()) as u32)
        {
            self.open_view(Box::new(AssemblyView::new(
                Decoder::with_ip(
                    if file_header.machine.get(LittleEndian) == pe::IMAGE_FILE_MACHINE_I386 {
                        32
                    } else {
                        64
                    },
                    instruction_data,
                    va,
                    DecoderOptions::NONE,
                )
                .iter()
                .take(50)
                .collect(),
            )));
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if let Some(go_to_assembly_va) = &self.state.go_to_assembly_va {
            self.open_assembly_view(*go_to_assembly_va);
            self.state.go_to_assembly_va = None;
        }
        DockArea::new(&mut self.tree).show(
            ctx,
            &mut TabViewer {
                state: &mut self.state,
            },
        );
    }
}

struct TabViewer<'a> {
    state: &'a mut AppState,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Box<dyn AppView>;

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        tab.ui(&mut self.state, ui);
    }

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.title().into()
    }
}

trait AppView {
    fn title(&self) -> String;

    fn ui(&mut self, state: &mut AppState, ui: &mut Ui);
}

#[derive(Default)]
struct AppState {
    go_to_assembly_va: Option<u64>,
}
