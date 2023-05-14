use std::fs;
use std::fs::File;
use iced_x86::{Decoder, DecoderOptions, Encoder, FlowControl, Formatter, Instruction, NasmFormatter, OpKind};
use iced_x86::CC_e::e;

use object::read::coff::CoffHeader;
use object::{LittleEndian, pe};
use object::read::pe::{ImageNtHeaders, ImageOptionalHeader};

fn main() -> anyhow::Result<()> {
    // read file
    let in_data = fs::read("C:\\Program Files (x86)\\Overwatch\\_retail_\\Overwatch.exe.tmp").unwrap();
    let in_data = in_data.as_slice();
    // decode headers
    let dos_header = pe::ImageDosHeader::parse(in_data)?;
    let mut nt_header_offset = dos_header.nt_headers_offset().into();
    let rich_header = object::read::pe::RichHeaderInfo::parse(in_data, nt_header_offset);
    let (nt_headers, data_directories) = pe::ImageNtHeaders64::parse(in_data, &mut nt_header_offset)?;
    let file_header = nt_headers.file_header();
    let optional_header = nt_headers.optional_header();
    // create writer
    let mut out_data = Vec::new();
    let mut writer = object::write::pe::Writer::new(
        nt_headers.is_type_64(),
        optional_header.section_alignment(),
        optional_header.file_alignment(),
        &mut out_data,
    );
    // reserve headers
    writer.reserve_dos_header_and_stub();
    if let Some(in_rich_header) = rich_header.as_ref() {
        writer.reserve(in_rich_header.length as u32 + 8, 4);
    }
    writer.reserve_nt_headers(data_directories.len());
    // copy data directories that don't have special handling
    let cert_dir = data_directories
        .get(pe::IMAGE_DIRECTORY_ENTRY_SECURITY)
        .map(pe::ImageDataDirectory::address_range);
    let reloc_dir = data_directories
        .get(pe::IMAGE_DIRECTORY_ENTRY_BASERELOC)
        .map(pe::ImageDataDirectory::address_range);
    for (i, dir) in data_directories.iter().enumerate() {
        if dir.virtual_address.get(LittleEndian) == 0
            || i == pe::IMAGE_DIRECTORY_ENTRY_SECURITY
            || i == pe::IMAGE_DIRECTORY_ENTRY_BASERELOC
        {
            continue;
        }
        writer.set_data_directory(i, dir.virtual_address.get(LittleEndian), dir.size.get(LittleEndian));
    }
    // reserve section headers
    let sections = file_header.sections(in_data, nt_header_offset)?;
    let mut sections_index = Vec::new();
    for (index, section) in sections.iter().enumerate() {
        if reloc_dir == Some(section.pe_address_range()) {
            continue;
        }
        sections_index.push(index + 1);
    }
    let mut sections_len = sections_index.len();
    if reloc_dir.is_some() {
        sections_len += 1;
    }
    writer.reserve_section_headers(sections_len as u16);
    // transform sections
    let mut sections_data = Vec::new();
    for index in &sections_index {
        let section = sections.section(*index)?;
        let range = writer.reserve_section(
            section.name,
            section.characteristics.get(LittleEndian),
            section.virtual_size.get(LittleEndian),
            section.size_of_raw_data.get(LittleEndian),
        );
        let mut section_data = section.pe_data(in_data)?.to_vec();
        if &section.name == b".text\0\0\0" {
            cfo(&mut Decoder::with_ip(64, section.pe_data(in_data)?, section.virtual_address.get(LittleEndian) as u64, DecoderOptions::NONE), optional_header.address_of_entry_point() as u64, &mut section_data)
        }
        sections_data.push((range.file_offset, section_data));
    }
    // reserve reloc section
    if reloc_dir.is_some() {
        let mut blocks = data_directories
            .relocation_blocks(in_data, &sections)?
            .unwrap();
        while let Some(block) = blocks.next()? {
            for reloc in block {
                writer.add_reloc(reloc.virtual_address, reloc.typ);
            }
        }
        writer.reserve_reloc_section();
    }
    // reserve certificate table
    if let Some((_, size)) = cert_dir {
        writer.reserve_certificate_table(size);
    }
    // encode headers
    writer.write_dos_header_and_stub()?;
    if let Some(in_rich_header) = rich_header.as_ref() {
        writer.write_align(4);
        writer.write(&in_data[in_rich_header.offset..][..in_rich_header.length + 8]);
    }
    writer.write_nt_headers(object::write::pe::NtHeaders {
        machine: file_header.machine.get(LittleEndian),
        time_date_stamp: file_header.time_date_stamp.get(LittleEndian),
        characteristics: file_header.characteristics.get(LittleEndian),
        major_linker_version: optional_header.major_linker_version(),
        minor_linker_version: optional_header.minor_linker_version(),
        address_of_entry_point: optional_header.address_of_entry_point(),
        image_base: optional_header.image_base(),
        major_operating_system_version: optional_header.major_operating_system_version(),
        minor_operating_system_version: optional_header.minor_operating_system_version(),
        major_image_version: optional_header.major_image_version(),
        minor_image_version: optional_header.minor_image_version(),
        major_subsystem_version: optional_header.major_subsystem_version(),
        minor_subsystem_version: optional_header.minor_subsystem_version(),
        subsystem: optional_header.subsystem(),
        dll_characteristics: optional_header.dll_characteristics(),
        size_of_stack_reserve: optional_header.size_of_stack_reserve(),
        size_of_stack_commit: optional_header.size_of_stack_commit(),
        size_of_heap_reserve: optional_header.size_of_heap_reserve(),
        size_of_heap_commit: optional_header.size_of_heap_commit(),
    });
    // encode section headers and sections
    writer.write_section_headers();
    for (offset, data) in sections_data {
        writer.write_section(offset, &data);
    }
    // encode reloc section
    writer.write_reloc_section();
    // encode certificate table
    if let Some((address, size)) = cert_dir {
        writer.write_certificate_table(&in_data[address as usize..][..size as usize]);
    }
    // write to file
    fs::write("C:\\Program Files (x86)\\Overwatch\\_retail_\\Overwatch.exe", out_data)?;

    Ok(())
}

fn cfo(decoder: &mut Decoder, entry: u64, data: &mut [u8]) {
    let base = decoder.ip();
    // go to entry
    decoder.set_ip(entry);
    decoder.set_position((entry - base) as usize).unwrap();
    // decode
    let mut instruction = Instruction::default();
    let mut cfo_branch = 0;
    let mut cfo_branch_target = 0;
    while decoder.can_decode() {
        decoder.decode_out(&mut instruction);
        if cfo_branch_target != 0 {
            if decoder.ip() >= cfo_branch_target {
                if (instruction.ip()..decoder.ip()).contains(&cfo_branch_target) {
                    for i in (cfo_branch - base)..(cfo_branch_target - base) {
                        data[i as usize] = 0x90;
                    }
                    decoder.set_ip(cfo_branch_target as u64);
                    decoder.set_position((cfo_branch_target - base) as usize).unwrap();
                    cfo_branch = 0;
                    cfo_branch_target = 0;
                } else {
                    decoder.set_ip(cfo_branch as u64);
                    decoder.set_position((cfo_branch - base) as usize).unwrap();
                    cfo_branch = 0;
                    cfo_branch_target = 0;
                }
            }
        } else {
            match instruction.flow_control() {
                FlowControl::UnconditionalBranch => if instruction.op0_kind() == OpKind::NearBranch64 && instruction.len() == 2 {
                    cfo_branch = decoder.ip();
                    cfo_branch_target = instruction.near_branch_target();
                }
                FlowControl::ConditionalBranch => if instruction.op0_kind() == OpKind::NearBranch64 && instruction.len() == 2 {
                    cfo_branch = decoder.ip();
                    cfo_branch_target = instruction.near_branch_target();
                }
                _ => {}
            }
        }
    }
}
