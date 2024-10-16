use crate::context::image::{ImageProvider, ImageProviderExt, ImageSection, Perms};
use crate::VarNode;
use object::elf::{SHF_EXECINSTR, SHF_WRITE};
use object::{Architecture, Endianness, File, Object, ObjectSection, SectionFlags, SectionKind};
use std::cmp::{max, min};
use std::io::empty;
use std::iter::once;

impl<'a> ImageProvider for File<'a> {
    fn load(&self, vn: &VarNode, output: &mut [u8]) -> usize {
        let mut written = 0;
        output.fill(0);
        let output_start_addr = vn.offset as usize;
        let output_end_addr = output_start_addr + vn.size;
        if let Some(x) = self.sections().find(|s| {
            output_start_addr > s.address() as usize
                && output_start_addr < (s.address() + s.size()) as usize
        }) {
            if let Ok(data) = x.data() {
                let input_start_addr = x.address() as usize;
                let input_end_addr = input_start_addr + data.len();
                let start_addr = max(input_start_addr, output_start_addr);
                let end_addr = max(min(input_end_addr, output_end_addr), start_addr);
                if end_addr > start_addr {
                    let i_s = start_addr - x.address() as usize;
                    let i_e = end_addr - x.address() as usize;
                    let o_s = start_addr - vn.offset as usize;
                    let o_e = end_addr - vn.offset as usize;
                    let out_slice = &mut output[o_s..o_e];
                    let in_slice = &data[i_s..i_e];
                    out_slice.copy_from_slice(in_slice);
                    written += end_addr - start_addr;
                }
            }
        }
        written
    }

    fn has_full_range(&self, vn: &VarNode) -> bool {
        self.sections().any(|s| {
            s.address() <= vn.offset && (s.address() + s.size()) >= (vn.offset + vn.size as u64)
        })
    }
}

impl<'a> ImageProviderExt for File<'a> {
    fn get_section_info(&self) -> impl Iterator<Item = ImageSection> {
        self.sections().filter_map(|s| {
            if let Ok(data) = s.data() {
                Some(ImageSection {
                    data,
                    base_address: s.address() as usize,
                    perms: map_sec_kind(&s.kind()),
                })
            } else {
                None
            }
        })
    }
}

pub fn map_gimli_architecture(file: &File) -> Option<&'static str> {
    match &file.architecture() {
        Architecture::Unknown => None,
        Architecture::Aarch64 => match file.endianness() {
            Endianness::Little => Some("AARCH64:LE:64:v8A"),
            Endianness::Big => Some("AARCH64:BE:64:v8A"),
        },
        Architecture::Aarch64_Ilp32 => match file.endianness() {
            Endianness::Little => Some("AARCH64:LE:32:ilp32"),
            Endianness::Big => Some("AARCH64:BE:32:ilp32"),
        },
        Architecture::Arm => match file.endianness() {
            Endianness::Little => Some("ARM:LE:32:v8"),
            Endianness::Big => Some("ARM:BE:32:v8"),
        },
        Architecture::I386 => Some("x86:LE:32:default"),
        Architecture::X86_64 => Some("x86:LE:64:default"),
        Architecture::PowerPc64 => match file.endianness() {
            Endianness::Little => Some("PowerPC:LE:64:default"),
            Endianness::Big => Some("PowerPC:BE:64:default"),
        },
        Architecture::Xtensa => match file.endianness() {
            Endianness::Little => Some("Xtensa:LE:32:default"),
            Endianness::Big => Some("Xtensa:BE:32:default"),
        },
        _ => None,
    }
}

fn map_sec_kind(kind: &SectionKind) -> Perms {
    match kind {
        SectionKind::Unknown => Perms::RWX,
        SectionKind::Text => Perms::RX,
        SectionKind::Data => Perms::RW,
        SectionKind::ReadOnlyData => Perms::R,
        SectionKind::ReadOnlyDataWithRel => Perms::R,
        SectionKind::ReadOnlyString => Perms::R,
        SectionKind::UninitializedData => Perms::RW,
        _ => Perms::NONE,
    }
}
