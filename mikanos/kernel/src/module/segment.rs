use core::arch::asm;

#[repr(transparent)]
#[derive(Clone, Copy)]
struct SegmentDescriptor(u64);

#[repr(transparent)]
#[derive(Clone, Copy)]
struct DescriptorType(u8);
impl DescriptorType {
    const READ_WRITE: Self = Self(2);
    const EXECUTE_READ: Self = Self(10);
}

const GDT_ENTRY_COUNT: usize = 3;
const GDT_LIMIT: u16 = (core::mem::size_of::<[SegmentDescriptor; GDT_ENTRY_COUNT]>() - 1) as u16;
static mut GDT: [SegmentDescriptor; GDT_ENTRY_COUNT] = [SegmentDescriptor(0); GDT_ENTRY_COUNT];

fn set_segment(
    desc: &mut SegmentDescriptor,
    descriptor_type: DescriptorType,
    descriptor_privilege_level: u32,
    base: u32,
    limit: u32,
) {
    desc.0 = 0;

    /* base_low, base_middle, base_high */
    desc.0 |= ((base & 0xffff) as u64) << 16;
    desc.0 |= (((base >> 16) & 0xff) as u64) << 32;
    desc.0 |= (((base >> 24) & 0xff) as u64) << 56;

    /* limit_low, limit_high */
    desc.0 |= (limit & 0xffff) as u64;
    desc.0 |= (((limit >> 16) & 0x0f) as u64) << 48;

    /* type */
    desc.0 |= ((descriptor_type.0 & 0x0f) as u64) << 40;

    /* system_segment = 1: コードまたはデータセグメント */
    desc.0 |= 1 << 44;

    /* descriptor_privilege_level */
    desc.0 |= ((descriptor_privilege_level & 0x03) as u64) << 45;

    /* present = 1 */
    desc.0 |= 1 << 47;

    /* available = 0なので設定不要 */

    /* granularity = 1 */
    desc.0 |= 1 << 55;
}

/**
*   @brief コードセグメントディスクリプタを設定
*/
fn set_code_segment(
    desc: &mut SegmentDescriptor,
    descriptor_type: DescriptorType,
    descriptor_privilege_level: u32,
    base: u32,
    limit: u32,
) {
    set_segment(
        desc,
        descriptor_type,
        descriptor_privilege_level,
        base,
        limit,
    );

    /* long_mode = 1 */
    desc.0 |= 1 << 53;

    /* default_operation_size = 0なので設定不要 */
}

/**
*   @brief データセグメントディスクリプタを設定
*/
fn set_data_segment(
    desc: &mut SegmentDescriptor,
    descriptor_type: DescriptorType,
    descriptor_privilege_level: u32,
    base: u32,
    limit: u32,
) {
    set_segment(
        desc,
        descriptor_type,
        descriptor_privilege_level,
        base,
        limit,
    );

    /* long_mode = 0なので設定不要 */

    /* default_operation_size = 1（32ビットスタックセグメント） */
    desc.0 |= 1 << 54;
}

fn load_gdt(limit: u16, offset: u64) {
    #[repr(C, packed)]
    struct Gdtr {
        limit: u16,
        offset: u64,
    }

    let gdtr = Gdtr { limit, offset };

    unsafe {
        asm!(
            "lgdt [{}]",
            in(reg) &gdtr,
            options(nostack, preserves_flags),
        );
    }
}

/**
*   @brief GDTを構築してロード
*/
pub fn setup_segments() {
    unsafe {
        /* gdt[0]: NULLディスクリプタ */
        GDT[0].0 = 0;

        /* gdt[1]: 64ビットカーネルコード */
        set_code_segment(&mut GDT[1], DescriptorType::EXECUTE_READ, 0, 0, 0xfffff);

        /* gdt[2]: カーネルデータ */
        set_data_segment(&mut GDT[2], DescriptorType::READ_WRITE, 0, 0, 0xfffff);

        let offset = core::ptr::addr_of!(GDT) as u64;

        load_gdt(GDT_LIMIT, offset);
    }
}
