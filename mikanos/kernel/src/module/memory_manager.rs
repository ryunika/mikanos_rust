use crate::module::err::{KError, KResult};
use crate::module::memory_map::{is_available, MemoryDescriptor, MemoryMap, UEFI_PAGE_SIZE};

/* 1フレームは4 KiB */
const BYTES_PER_FRAME: usize = 4096;

/* 教科書と同じく、最大128 GiB分を管理 */
const MAX_PHYSICAL_MEMORY_BYTES: usize = 128 * 1024 * 1024 * 1024;
const FRAME_COUNT: usize = MAX_PHYSICAL_MEMORY_BYTES / BYTES_PER_FRAME;

const BITS_PER_MAP_LINE: usize = u64::BITS as usize;
const MAP_LINE_COUNT: usize = FRAME_COUNT / BITS_PER_MAP_LINE;

static mut MEMORY_MANAGER: BitmapMemoryManager = BitmapMemoryManager::new();

struct FrameID {
    id: usize,
}

struct BitmapMemoryManager {
    alloc_map: [u64; MAP_LINE_COUNT],
    range_begin: FrameID,
    range_end: FrameID,
}

impl FrameID {
    fn offset(&self, count: usize) -> Self {
        Self {
            id: self.id + count,
        }
    }
}

impl BitmapMemoryManager {
    const fn new() -> Self {
        Self {
            alloc_map: [0; MAP_LINE_COUNT],
            range_begin: FrameID { id: 0 },
            range_end: FrameID { id: FRAME_COUNT },
        }
    }

    fn initialize(&mut self, memory_map: &MemoryMap) {
        let map_begin = memory_map.buffer as usize;
        let map_end = map_begin + memory_map.map_size as usize;

        let mut iter = map_begin;
        let mut available_end = 0;

        while iter < map_end {
            /* ブートローダーが渡した有効なメモリ記述子を参照する */
            let desc = unsafe { &*(iter as *const MemoryDescriptor) };

            if available_end < desc.physical_start {
                let start_frame = FrameID {
                    id: available_end / BYTES_PER_FRAME,
                };

                let num_frames = (desc.physical_start - available_end) / BYTES_PER_FRAME;

                self.mark_allocated(start_frame, num_frames);
            }

            let region_size = desc.number_of_pages as usize * UEFI_PAGE_SIZE;

            let physical_end = desc.physical_start + region_size;

            if is_available(desc.memory_type) {
                available_end = physical_end;
            } else {
                let start_frame = FrameID {
                    id: desc.physical_start / BYTES_PER_FRAME,
                };

                let num_frames = region_size / BYTES_PER_FRAME;

                self.mark_allocated(start_frame, num_frames);
            }

            iter += memory_map.descriptor_size as usize;
        }

        self.set_memory_range(
            FrameID { id: 1 },
            FrameID {
                id: available_end / BYTES_PER_FRAME,
            },
        );
    }

    fn get_bit(&self, frame: FrameID) -> bool {
        let line_index = frame.id / BITS_PER_MAP_LINE;
        let bit_index = frame.id % BITS_PER_MAP_LINE;
        let mask = 1_u64 << bit_index;

        (self.alloc_map[line_index] & mask) != 0
    }

    fn set_bit(&mut self, frame: FrameID, allocated: bool) {
        let line_index = frame.id / BITS_PER_MAP_LINE;
        let bit_index = frame.id % BITS_PER_MAP_LINE;
        let mask = 1_u64 << bit_index;

        if allocated {
            /* 対象ビットを1にする */
            self.alloc_map[line_index] |= mask;
        } else {
            /* 対象ビットを0にする */
            self.alloc_map[line_index] &= !mask;
        }
    }

    fn count_free_frames(&self, start: &FrameID, num_frames: usize) -> KResult<usize> {
        let mut found = 0;

        while found < num_frames {
            let frame = start.offset(found);

            if frame.id >= self.range_end.id {
                return Err(KError::NoEnoughMemory);
            }

            if self.get_bit(frame) {
                break;
            }

            found += 1;
        }

        Ok(found)
    }

    pub fn allocate(&mut self, num_frames: usize) -> KResult<FrameID> {
        let mut start = self.range_begin.offset(0);
        let mut candidate = None;

        while start.id < self.range_end.id {
            let found = match self.count_free_frames(&start, num_frames) {
                Ok(count) => count,
                Err(error) => return Err(error),
            };

            if found == num_frames {
                candidate = Some(start);
                break;
            }

            /* 使用中だったフレームの次から探し直す */
            start = start.offset(found + 1);
        }

        let start_frame = match candidate {
            Some(frame) => frame,
            None => return Err(KError::NoEnoughMemory),
        };

        for i in 0..num_frames {
            self.set_bit(start_frame.offset(i), true);
        }

        Ok(start_frame)
    }

    pub fn free(&mut self, start_frame: FrameID, num_frames: usize) {
        for i in 0..num_frames {
            self.set_bit(start_frame.offset(i), false);
        }
    }

    pub fn mark_allocated(&mut self, start_frame: FrameID, num_frames: usize) {
        for i in 0..num_frames {
            self.set_bit(start_frame.offset(i), true);
        }
    }

    pub fn set_memory_range(&mut self, begin: FrameID, end: FrameID) {
        self.range_begin = begin;
        self.range_end = end;
    }
}

pub fn initialize(memory_map: &MemoryMap) {
    unsafe {
        MEMORY_MANAGER.initialize(memory_map);
    }
}
