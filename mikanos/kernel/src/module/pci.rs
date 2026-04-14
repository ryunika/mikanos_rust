use core::arch::asm;
use crate::module::err::{KResult,KError};


/**
*   @brief MSI ケーパビリティヘッダ（C++ の union header に相当）
*/
#[derive(Clone, Copy)]
struct MsiCapabilityHeader {
    data: u32,
}

impl MsiCapabilityHeader {
    const CAP_ID_SHIFT: u32                  = 0;
    const NEXT_PTR_SHIFT: u32                = 8;
    const MSI_ENABLE_SHIFT: u32              = 16;
    const MULTI_MSG_CAPABLE_SHIFT: u32       = 17;
    const MULTI_MSG_ENABLE_SHIFT: u32        = 20;
    const ADDR_64_CAPABLE_SHIFT: u32         = 23;
    const PER_VECTOR_MASK_CAPABLE_SHIFT: u32 = 24;

    const BYTE_MASK: u32                     = 0xff;
    const BIT_MASK: u32                      = 0x1;
    const THREE_BIT_MASK: u32               = 0b111;

    fn cap_id(self) -> u8            { ((self.data >> Self::CAP_ID_SHIFT) & Self::BYTE_MASK) as u8 }
    fn next_ptr(self) -> u8          { ((self.data >> Self::NEXT_PTR_SHIFT) & Self::BYTE_MASK) as u8 }
    fn msi_enable(self) -> bool      { (self.data >> Self::MSI_ENABLE_SHIFT) & Self::BIT_MASK != 0 }
    fn multi_msg_capable(self) -> u8 { ((self.data >> Self::MULTI_MSG_CAPABLE_SHIFT) & Self::THREE_BIT_MASK) as u8 }
    fn multi_msg_enable(self) -> u8  { ((self.data >> Self::MULTI_MSG_ENABLE_SHIFT) & Self::THREE_BIT_MASK) as u8 }
    fn addr_64_capable(self) -> bool { (self.data >> Self::ADDR_64_CAPABLE_SHIFT) & Self::BIT_MASK != 0 }
    fn per_vector_mask_capable(self) -> bool { (self.data >> Self::PER_VECTOR_MASK_CAPABLE_SHIFT) & Self::BIT_MASK != 0 }

    fn set_msi_enable(&mut self, val: bool) {
        self.data = (self.data & !(Self::BIT_MASK << Self::MSI_ENABLE_SHIFT))
                  | ((val as u32) << Self::MSI_ENABLE_SHIFT);
    }
    fn set_multi_msg_enable(&mut self, val: u8) {
        self.data = (self.data & !(Self::THREE_BIT_MASK << Self::MULTI_MSG_ENABLE_SHIFT))
                  | (((val as u32) & Self::THREE_BIT_MASK) << Self::MULTI_MSG_ENABLE_SHIFT);
    }
}

/**
*   @brief MSI ケーパビリティ構造
*   @note ケーパビリティ構造は 64 ビットサポートの有無などで亜種が沢山ある．
*         この構造体は各亜種に対応するために最大の亜種に合わせてメンバを定義してある．
*/
#[repr(C, packed)]
struct MSICapability {
    header: MsiCapabilityHeader,
    msg_addr: u32,
    msg_upper_addr: u32,
    msg_data: u32,
    mask_bits: u32,
    pending_bits: u32,
}

/* PCIデバイスのクラスコード */
#[derive(Copy,Clone)]
pub struct ClassCode {
    base: u8,
    sub: u8,
    interface: u8,
}

impl ClassCode {
    pub fn same(&self,base: u8, sub: u8, interface: u8) -> bool {
        self.base == base && self.sub == sub && self.interface == interface
    }
}

#[derive(Copy,Clone)]
pub struct Device {
    bus: u8,
    device: u8,
    function: u8,
    header_type: u8,
    pub class_code: ClassCode,
}

pub struct Pci {
    pub num_device: usize,
    pub devices: [Device; 32],
}

/* CONFIG_ADDRESSレジスタのIOポートアドレス */
const CONFIG_ADDRESS: u16 = 0x0cf8;

/* CONFIG_DATAレジスタのIOポートアドレス */
const CONFIG_DATA: u16 = 0x0cfc;

/* 無効なベンダーID */
const VENDOR_ID_INVALID: u16 = 0xffff;

impl Pci {

pub fn new() -> Self {
    Self {
        num_device: 0,
        devices: [Device {
            bus: 0,
            device: 0,
            function: 0,
            header_type: 0,
            class_code: ClassCode {
                base: 0,
                sub: 0,
                interface: 0,
            },
        }; 32],
    }
}

/**
* @brief 発見したデバイスの情報を表示する
*/
pub fn show_device(&self,console: &mut impl core::fmt::Write) {

    for i in 0..self.num_device {
        let dev = &self.devices[i];
        let vendor_id = read_vendor_id(dev.bus, dev.device, dev.function);
        let class_code = read_class_code(dev.bus, dev.device, dev.function);

        write!(console, "{}:{}:{}: vend {:04x}, class {:02x},{:02x},{:02x}, head {:02x}\n",
            dev.bus, dev.device, dev.function,
            vendor_id,
            class_code.base,
            class_code.sub,
            class_code.interface,
            dev.header_type);
    }
}

/**
* @brief devices[num_device] に情報を書き込み num_device をインクリメントする
*/
fn add_device(& mut self, dev: Device) -> KResult<()> {
    if self.num_device == self.devices.len() {
        return Err(KError::Full);
    }

    self.devices[self.num_device] = dev;
    self.num_device += 1;
    Ok(())
}



/**
* @brief 指定のファンクションを devices に追加する
* @note もし PCI-PCI ブリッジなら，セカンダリバスに対し ScanBus を実行する
*/
fn scan_function(&mut self,bus: u8, device: u8, function: u8) -> KResult<()> {
    let header_type = read_header_type(bus, device, function);
    let class_code = read_class_code(bus, device, function);

    let dev: Device = Device {
        bus,
        device,
        function,
        header_type,
        class_code,
    };
    if let Err(err) = self.add_device(dev) {
        return Err(err);
    }

    

    if class_code.base == 0x06u8 && class_code.sub == 0x04u8 {
        /* PCI-PCI ブリッジ */
        let bus_numbers = read_bus_numbers(bus, device, function);
        let secondary_bus:u8 = (bus_numbers >> 8) as u8 & 0xffu8;
        return self.scan_bus(secondary_bus);
    }

    Ok(())
}

/**
* @brief PCI デバイスをすべて探索し devices に格納する
* @note 発見したデバイスの数を num_devices に設定する
*/
pub fn scan_all_bus(&mut self) -> KResult<()> {
    self.num_device = 0;

    let header_type = read_header_type(0, 0, 0);
    if is_single_function_device(header_type) {
        return self.scan_bus(0);
    }

    for function in 1..8 {
        if read_vendor_id(0, 0, function) == VENDOR_ID_INVALID {
            continue;
        }

        self.scan_bus(function)?;
    }
    
    Ok(())
}

/**
* @brief 指定のバス番号の各デバイスをスキャンする
* @note 有効なデバイスを見つけたら ScanDevice を実行する．
*/
fn scan_bus(&mut self,bus: u8) -> KResult<()> {
    for device in 0..32 {
        if read_vendor_id(bus, device, 0) == VENDOR_ID_INVALID {
            continue;
        }

        if let Err(err) = self.scan_device(bus, device) {
            return Err(err);
        }
    }
    return Ok(());
}

/**
* @brief 指定のデバイス番号の各ファンクションをスキャンする
* @note 有効なファンクションを見つけたら ScanFunction を実行する
*/
fn scan_device(&mut self,bus: u8, device: u8) -> KResult<()> {
    self.scan_function(bus, device, 0)?;

    let header_type = read_header_type(bus, device, 0);
    if is_single_function_device(header_type) {
        return Ok(());
    }

    for function in 1..8 {
        if read_vendor_id(bus, device, function) == VENDOR_ID_INVALID {
            continue;
        }

        if let Err(err) = self.scan_function(bus, device, function) {
            return Err(err);
        }
    }

    Ok(())
}

}

/**
* @brief 単一ファンクションか
*/
fn is_single_function_device(header_type: u8) -> bool {
    (header_type & 0x80u8) == 0
}

/**
* @brief コンフィグレーション空間からヘッダタイプを読み取る
*/
fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    const HEADER_TYPE_OFFSET: u8 = 0x0c;
    let address: u32 = make_address(bus, device, function, HEADER_TYPE_OFFSET);
    write_address(address);
    (read_data() >> 16) as u8 &0xffu8
}

/**
* @brief コンフィグレーション空間からクラスコードを読み取る
*/
fn read_class_code(bus: u8, device: u8, function: u8) -> ClassCode {
    const CLASS_CODE_OFFSET: u8 = 0x08;
    let address: u32 = make_address(bus, device, function, CLASS_CODE_OFFSET);
    write_address(address);
    let reg: u32 = read_data();

    ClassCode {
        base: (reg >> 24) as u8 & 0xffu8,
        sub: (reg >> 16) as u8 & 0xffu8,
        interface: (reg >> 8) as u8 & 0xffu8,
    }
}

fn read_bus_numbers(bus: u8, device: u8, function: u8) -> u32 {
    const BUS_NUMBERS_OFFSET: u8 = 0x18;
    let address: u32 = make_address(bus, device, function, BUS_NUMBERS_OFFSET);
    write_address(address);
    read_data()
}

/**
* @brief コンフィグレーション空間からベンダIDを読み取る
*/
fn read_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    const VENDOR_ID_OFFSET: u8 = 0x00;
    let address: u32 = make_address(bus, device, function, VENDOR_ID_OFFSET);
    write_address(address);
    (read_data() & 0xffff) as u16
}


/**
* @brief CONFIG_ADDRESS用のアドレスを作成する
*/
fn make_address(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    const ENABLE_BIT_SHIFT : u32 = 31;
    const BUS_NUM_SHIFT: u32 = 16;
    const DEVICE_NUM_SHIFT: u32 = 11;
    const FUNCTION_NUM_SHIFT: u32 = 8;

    let mut address: u32 = 0;

    address |= ((1 as u32) << ENABLE_BIT_SHIFT) as u32;
    address |= ((bus as u32) << BUS_NUM_SHIFT) as u32;
    address |= ((device as u32) << DEVICE_NUM_SHIFT) as u32;
    address |= ((function as u32) << FUNCTION_NUM_SHIFT) as u32;
    address |= ((offset & 0xfcu8) as u32);

    address
}

/**
* @brief CONFIG_ADDRESSレジスタにアドレスを書き込む
*/
fn write_address(address: u32) {
    io_out32(CONFIG_ADDRESS, address);
}

/**
* @brief CONFIG_DATAレジスタにデータを書き込む
*/
fn write_data(data: u32) {
    io_out32(CONFIG_DATA, data);
}

/**
* @brief CONFIG_DATAレジスタからデータを読み出す
*/
fn read_data() -> u32 {
    io_in32(CONFIG_DATA)
}

fn io_out32(port: u16, data:u32) {
    unsafe {
        asm!(
            "out dx, eax",
            in("dx") port,
            in("eax") data,
            options(nomem, nostack, preserves_flags),
        );
    }
}

fn io_in32(port: u16) -> u32 {
    let value: u32;
    unsafe {
        asm!(
            "in eax, dx",
            out("eax") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags),
        );
    }
    value
}

fn read_conf_reg(dev: &Device, reg_addr: u8) -> u32 {
    let address: u32 = make_address(dev.bus, dev.device, dev.function, reg_addr);
    write_address(address);
    read_data()
}

fn write_conf_reg(dev: &Device, reg_addr: u8, value: u32) {
    let address = make_address(dev.bus, dev.device, dev.function, reg_addr);
    write_data(value);
}

fn calc_bar_address(bar_index: usize) -> u8 {
    (0x10 + 4 * bar_index) as u8
}

pub fn read_bar(dev: &Device, bar_index: usize) -> KResult<u64> {
    /* BAR0からBAR5まで */
    if bar_index >= 6 {
        return Err(KError::IndexOutOfRange);
    }

    let bar_addr = calc_bar_address(bar_index);
    let bar_low = read_conf_reg(dev, bar_addr);

    if bar_low & 0x4u32 == 0 {
        /* 32bitアドレス */
        Ok(bar_low as u64)
    } else {
        /* 64bitアドレス */
        if bar_index == 5 {
            return Err(KError::IndexOutOfRange);
        }

        let bar_high = read_conf_reg(dev, bar_addr + 4);

        let bar: u64 = ((bar_high as u64) << 32) | (bar_low as u64);
        Ok(bar)
    }

}


 /** @brief 指定された MSI ケーパビリティ構造を読み取る
   *
   * @param dev  MSI ケーパビリティを読み込む PCI デバイス
   * @param cap_addr  MSI ケーパビリティレジスタのコンフィグレーション空間アドレス
   */
//   MSICapability ReadMSICapability(const Device& dev, uint8_t cap_addr) {
//     MSICapability msi_cap{};

//     msi_cap.header.data = ReadConfReg(dev, cap_addr);
//     msi_cap.msg_addr = ReadConfReg(dev, cap_addr + 4);

//     uint8_t msg_data_addr = cap_addr + 8;
//     if (msi_cap.header.bits.addr_64_capable) {
//       msi_cap.msg_upper_addr = ReadConfReg(dev, cap_addr + 8);
//       msg_data_addr = cap_addr + 12;
//     }

//     msi_cap.msg_data = ReadConfReg(dev, msg_data_addr);

//     if (msi_cap.header.bits.per_vector_mask_capable) {
//       msi_cap.mask_bits = ReadConfReg(dev, msg_data_addr + 4);
//       msi_cap.pending_bits = ReadConfReg(dev, msg_data_addr + 8);
//     }

//     return msi_cap;
//   }

/**
*   @brief 指定された MSI ケーパビリティ構造を読み取る
*/
fn read_msi_capability(
    dev: &Device,
    cap_addr: u8
) -> MSICapability {
    let msi_cap: MSICapability;

    msi_cap.header.data = read_conf_reg(dev, cap_addr);
    msi_cap.msg_addr = read_conf_reg(dev, cap_addr);

    if msi_cap.header.addr_64_capable() {
        msi_cap.msg_upper_addr = read_conf_reg(dev, cap_addr + 8);
        let msg_data_addr = cap_addr + 12;
    } else {
        let msg_data_addr = cap_addr + 8;
    }
    

    msi_cap.msg_data = read_conf_reg(dev, msg_data_addr);

    if msi_cap.header.per_vector_mask_capable() {
        msi_cap.mask_bits = read_conf_reg(dev, msg_data_addr + 4);
        msi_cap.pending_bits = read_conf_reg(dev, msg_data_addr + 8);
    }

    msi_cap
}

/** @brief 指定された MSI ケーパビリティ構造に書き込む
*
* @param dev  MSI ケーパビリティを読み込む PCI デバイス
* @param cap_addr  MSI ケーパビリティレジスタのコンフィグレーション空間アドレス
* @param msi_cap  書き込む値
*/
fn write_msi_capability(
    dev: &Device,
    cap_addr: u8,
    msi_cap: &MSICapability
) {
    write_conf_reg(dev, cap_addr, msi_cap.header.data);
    write_conf_reg(dev, cap_addr + 4, msi_cap.msg_addr);

    let msg_data_addr = if msi_cap.header.addr_64_capable() {
        write_conf_reg(dev, cap_addr + 8, msi_cap.msg_upper_addr);
        cap_addr + 12
    } else {
        cap_addr + 8
    };

    write_conf_reg(dev, msg_data_addr, msi_cap.msg_data);

    if msi_cap.header.per_vector_mask_capable() {
        write_conf_reg(dev, msg_data_addr + 4, msi_cap.mask_bits);
        write_conf_reg(dev, msg_data_addr + 8, msi_cap.pending_bits);
    }
    
}


/**
*   @brief 指定された MSI レジスタを設定する
*/
fn configure_msi_register(
    dev: &Device,
    cap_addr: u8,
    msg_addr: u32,
    msg_data: u32,
    num_vector_exponent: u32,
) -> KResult<()> {

    let mut msi_cap = read_msi_capability(dev, cap_addr);

    if msi_cap.header.multi_msg_capable() <=  num_vector_exponent as u8 {
        msi_cap.header.set_multi_msg_enable(msi_cap.header.multi_msg_capable());
    } else {
        msi_cap.header.set_multi_msg_enable(num_vector_exponent as u8);
    }

    msi_cap.header.set_msi_enable(true);
    msi_cap.msg_addr = msg_addr;
    msi_cap.msg_data = msg_data;

    write_msi_capability(dev, cap_addr, &msi_cap);

    Ok(())
}

/**
*   @brief 指定された MSI レジスタを設定する
*/
fn configure_msix_register(
    _dev: &Device,
    _cap_addr: u8,
    _msg_addr: u32,
    _msg_data: u32,
    _num_vector_exponent: u32,
) -> KResult<()> {
    Err(KError::NotImplemented)
}

fn read_capability_header(dev: &Device, cap_addr: u8) -> MsiCapabilityHeader {
    let mut header :MsiCapabilityHeader;
    header.data = read_conf_reg(dev, cap_addr);
    header
}

fn configure_msi(
    dev: &Device,
    msg_addr: u32,
    msg_data: u32,
    num_vector_exponent: u32,
) -> KResult<()> {
    const CAPABILITY_MSI: u8 = 0x05;
    const CAPABILITY_MSIX: u8 = 0x11;
    
    let mut cap_addr: u8 = read_conf_reg(dev, 0x34) as u8;

    let mut msi_cap_addr: u8 = 0;
    let mut msix_cap_addr: u8 = 0;
    while cap_addr != 0 {
        let header = read_capability_header(dev, cap_addr);
        if header.cap_id() == CAPABILITY_MSI {
            msi_cap_addr = cap_addr;
        } else if header.cap_id() == CAPABILITY_MSIX {
            msix_cap_addr = cap_addr;
        }
        cap_addr = header.next_ptr();
    }

    if msi_cap_addr != 0 {
        configure_msi_register(dev, msi_cap_addr, msg_addr, msg_data, num_vector_exponent)
    } else if msix_cap_addr != 0 {
        configure_msix_register(dev, msix_cap_addr, msg_addr, msg_data, num_vector_exponent)
    } else {
        Err(KError::NoPCIMSI)
    }
}


//   enum class MSITriggerMode {
//     kEdge = 0,
//     kLevel = 1
//   };

pub enum MSITriggerMode {
    Edge = 0,
    Level = 1
}

pub enum MSIDeliveryMode {
    Fixed          = 0b000,
    LowestPriority = 0b001,
    SMI            = 0b010,
    NMI            = 0b100,
    INIT           = 0b101,
    ExtINT         = 0b111,
}

pub fn configure_msi_fixed_destination(
    dev: &Device,
    apic_id: u8,
    trigger_mode: MSITriggerMode,
    delivery_mode: MSIDeliveryMode,
    vector: u8,
    num_vector_exponent: u32
) -> KResult<()> {
    let mut msg_addr: u32 = 0xfee00000_u32 | (apic_id << 12);
    let mut msg_data: u32 = delivery_mode << 8 | vector;

    if trigger_mode == MSITriggerMode::Level {
        msg_data |= 0xc000;
    }

    configure_msi(dev, msg_addr, msg_data, num_vector_exponent)
}