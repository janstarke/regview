use zerocopy::ByteSlice;
use memmap::Mmap;
use std::rc::Rc;
use std::ops::Deref;

pub struct MmapByteSlice {
    mmap: Rc<Mmap>,
    start: usize,
    end: usize,
}

impl MmapByteSlice {
    pub fn new(mmap: Mmap) -> Self {
        let end = mmap.len();
        Self {
            mmap: Rc::new(mmap),
            start: 0,
            end
        }
    }
}

impl Deref for MmapByteSlice {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.mmap[self.start..self.end]
    }
}

unsafe impl ByteSlice for MmapByteSlice {
    fn as_ptr(&self) -> *const u8 {
        (&self.mmap[self.start..self.end]).as_ptr()
    }
    fn split_at(self, mid: usize) -> (Self, Self) {
        (
            Self {
                mmap: Rc::clone(&self.mmap),
                start: self.start,
                end: mid
            }
            ,
            Self {
                mmap: Rc::clone(&self.mmap),
                start: mid,
                end: self.end

            }
        )
    }
}