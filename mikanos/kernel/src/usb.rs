/* C++ USB スタックへの FFI 宣言 */
extern "C" {
    fn usb_xhci_controller_new(mmio_base: usize) -> *mut u8;
    fn usb_xhci_controller_initialize(xhc: *mut u8) -> i32;
    fn usb_xhci_controller_run(xhc: *mut u8) -> i32;
    fn usb_xhci_controller_configure_port(xhc: *mut u8, port_num: u8) -> i32;
    fn usb_xhci_controller_process_event(xhc: *mut u8) -> i32;
    fn usb_xhci_controller_primary_event_ring_has_front(xhc: *mut u8) -> bool;
    fn usb_xhci_controller_max_ports(xhc: *mut u8) -> u8;
    fn usb_xhci_controller_register_mouse_observer(
        xhc: *mut u8,
        cb: extern "C" fn(buttons: u8, dx: i8, dy: i8),
    );
}

/* xHCI コントローラーの Rust ラッパー */
pub struct XhciController {
    ptr: *mut u8,
}

impl XhciController {
    /* コントローラーを生成・初期化・起動する */
    fn new(mmio_base: usize) -> Result<Self, i32> {
        unsafe {
            let ptr = usb_xhci_controller_new(mmio_base);
            let r = usb_xhci_controller_initialize(ptr);
            if r != 0 {
                return Err(r);
            }
            let r = usb_xhci_controller_run(ptr);
            if r != 0 {
                return Err(r);
            }
            Ok(XhciController { ptr })
        }
    }

    /* 全ポートを列挙して接続済みデバイスを設定する */
    pub fn configure_ports(&self) {
        unsafe {
            let max = usb_xhci_controller_max_ports(self.ptr);
            for i in 1..=max {
                usb_xhci_controller_configure_port(self.ptr, i);
            }
        }
    }

    /* マウスイベントのコールバックを登録する */
    pub fn register_mouse_observer(&self, cb: extern "C" fn(u8, i8, i8)) {
        unsafe {
            usb_xhci_controller_register_mouse_observer(self.ptr, cb);
        }
    }

    /* プライマリイベントリングにイベントが存在するか確認する */
    pub fn has_event(&self) -> bool {
        unsafe { usb_xhci_controller_primary_event_ring_has_front(self.ptr) }
    }

    /* イベントを1件処理する */
    pub fn process_event(&self) -> i32 {
        unsafe { usb_xhci_controller_process_event(self.ptr) }
    }
}

/* グローバルシングルトンとして xHCI コントローラーを管理するモジュール */
pub mod global {
    use super::XhciController;

    /* シングルスレッドカーネルのため Mutex なしで static mut を使用する */
    static mut CONTROLLER: Option<XhciController> = None;

    pub fn initialize(mmio_base: usize) -> Result<(), i32> {
        let c = XhciController::new(mmio_base)?;
        unsafe {
            CONTROLLER = Some(c);
        }
        Ok(())
    }

    pub fn controller() -> &'static XhciController {
        unsafe { CONTROLLER.as_ref().unwrap() }
    }

    pub fn is_initialized() -> bool {
        unsafe { CONTROLLER.is_some() }
    }
}
