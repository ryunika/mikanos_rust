use core::arch::asm;

const PAGE_SIZE_4K: u64 = 4096;
const PAGE_SIZE_2M: u64 = 512 * PAGE_SIZE_4K;
const PAGE_SIZE_1G: u64 = 512 * PAGE_SIZE_2M;

const PAGE_DIRECTORY_COUNT: usize = 64;

/* ページングエントリの属性 */
const PRESENT: u64 = 0x001;
const WRITABLE: u64 = 0x002;

/* ページディレクトリで2 MiBページを指定するビット */
const HUGE_PAGE: u64 = 0x080;

#[repr(C, align(4096))]
struct PageTable {
    entries: [u64; 512],
}

static mut PML4_TABLE: PageTable = PageTable { entries: [0; 512] };
static mut PDP_TABLE: PageTable = PageTable { entries: [0; 512] };

static mut PAGE_DIRECTORY: [PageTable; PAGE_DIRECTORY_COUNT] =
    [const { PageTable { entries: [0; 512] } }; PAGE_DIRECTORY_COUNT];

/**
*   @brief CR3レジスタにロードする
*/
fn set_cr3(value: u64) {
    unsafe {
        asm!(
            "mov cr3, {}",
            in(reg) value,
            options(nostack, preserves_flags),
        );
    }
}

pub fn setup_identity_page_table() {
    unsafe {
        PML4_TABLE.entries[0] = core::ptr::addr_of!(PDP_TABLE) as u64 | PRESENT | WRITABLE;

        for i_pdpt in 0..PAGE_DIRECTORY_COUNT {
            /* PDPの各エントリからページディレクトリを指す */
            PDP_TABLE.entries[i_pdpt] =
                core::ptr::addr_of!(PAGE_DIRECTORY[i_pdpt]) as u64 | PRESENT | WRITABLE;

            for i_pd in 0..512 {
                /* 2 MiB単位で仮想アドレスと物理アドレスを一致させる */
                PAGE_DIRECTORY[i_pdpt].entries[i_pd] = (i_pdpt as u64 * PAGE_SIZE_1G
                    + i_pd as u64 * PAGE_SIZE_2M)
                    | PRESENT
                    | WRITABLE
                    | HUGE_PAGE;
            }
        }

        /* 作成したページテーブルへ切り替える */
        set_cr3(core::ptr::addr_of!(PML4_TABLE) as u64);
    }
}
