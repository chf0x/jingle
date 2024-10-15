use crate::context::image::ImageProvider;
use crate::ffi::instruction::bridge::VarnodeInfoFFI;
use crate::VarNode;
use std::fmt::{Debug, Formatter};

pub(crate) struct ImageFFI<'a> {
    provider: Box<dyn ImageProvider + 'a>,
}

impl<'a> ImageFFI<'a> {
    pub(crate) fn new<T: ImageProvider + 'a>(provider: T) -> Self {
        Self {
            provider: Box::new(provider),
        }
    }
    fn load(&self, vn: &VarnodeInfoFFI, out: &mut [u8]) -> usize {
        self.provider.load(&VarNode::from(vn), out)
    }

    pub(crate) fn has_range(&self, vn: &VarNode) -> bool {
        self.provider.has_range(vn)
    }
}

#[cxx::bridge]
pub(crate) mod bridge {
    unsafe extern "C++" {
        type VarnodeInfoFFI = crate::ffi::instruction::bridge::VarnodeInfoFFI;
    }
    extern "Rust" {
        include!("jingle_sleigh/src/ffi/instruction.rs.h");
        type ImageFFI<'a>;
        fn load(self: &ImageFFI, vn: &VarnodeInfoFFI, out: &mut [u8]) -> usize;
    }
}
