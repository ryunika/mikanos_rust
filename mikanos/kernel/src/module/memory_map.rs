use core::ffi::c_void;

/* UEFIのページサイズ */
pub const UEFI_PAGE_SIZE: usize = 4096;

/* UEFI BOOT側と同じ定義を使う */
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryMap {
    pub buffer_size: u64,
    pub buffer: *mut c_void,
    pub map_size: u64,
    pub map_key: u64,
    pub descriptor_size: u64,
    pub descriptor_version: u32,
}

/* UEFI BOOT側と同じ定義を使う */
#[derive(Eq, PartialEq)]
#[warn(dead_code)]
#[repr(C)]
pub struct MemoryDescriptor {
    pub memory_type: u32,
    pub physical_start: usize,
    pub virtual_start: usize,
    pub number_of_pages: u64,
    pub attribute: u64,
}

/* UEFI BOOT側と同じ定義を使う */
#[warn(dead_code)]
#[derive(Eq, PartialEq, Debug)]
#[repr(C)]
pub enum MemoryType {
    KEfiReservedMemoryType,
    KEfiLoaderCode,
    KEfiLoaderData,
    KEfiBootServicesCode,
    KEfiBootServicesData,
    KEfiRuntimeServicesCode,
    KEfiRuntimeServicesData,
    KEfiConventionalMemory,
    KEfiUnusableMemory,
    KEfiACPIReclaimMemory,
    KEfiACPIMemoryNVS,
    KEfiMemoryMappedIO,
    KEfiMemoryMappedIOPortSpace,
    KEfiPalCode,
    KEfiPersistentMemory,
    KEfiMaxMemoryType,
}

impl TryFrom<u32> for MemoryType {
    type Error = u32;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::KEfiReservedMemoryType),
            1 => Ok(Self::KEfiLoaderCode),
            2 => Ok(Self::KEfiLoaderData),
            3 => Ok(Self::KEfiBootServicesCode),
            4 => Ok(Self::KEfiBootServicesData),
            5 => Ok(Self::KEfiRuntimeServicesCode),
            6 => Ok(Self::KEfiRuntimeServicesData),
            7 => Ok(Self::KEfiConventionalMemory),
            8 => Ok(Self::KEfiUnusableMemory),
            9 => Ok(Self::KEfiACPIReclaimMemory),
            10 => Ok(Self::KEfiACPIMemoryNVS),
            11 => Ok(Self::KEfiMemoryMappedIO),
            12 => Ok(Self::KEfiMemoryMappedIOPortSpace),
            13 => Ok(Self::KEfiPalCode),
            14 => Ok(Self::KEfiPersistentMemory),
            15 => Ok(Self::KEfiMaxMemoryType),
            _ => Err(value),
        }
    }
}

/* カーネルが再利用できるメモリタイプかを判定する */
pub fn is_available(memory_type: u32) -> bool {
    matches!(
        MemoryType::try_from(memory_type),
        Ok(MemoryType::KEfiBootServicesCode)
            | Ok(MemoryType::KEfiBootServicesData)
            | Ok(MemoryType::KEfiConventionalMemory)
    )
}
