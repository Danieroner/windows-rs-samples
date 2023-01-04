use windows::{
  core::*,
  Win32::Foundation::*,
  Win32::System::LibraryLoader::GetModuleHandleA,
  Win32::UI::WindowsAndMessaging::*,
};
use once_cell::sync::Lazy;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::{Arc, LockResult, Mutex};
use std::io::{self, Write};
use std::thread;

static COUNT: Lazy<Arc<Mutex<AtomicUsize>>> =
  Lazy::new(|| Arc::new(Mutex::new(AtomicUsize::new(0))));

fn main() -> Result<()> {
  unsafe {
    let instance = GetModuleHandleA(None)?;
    debug_assert!(instance.0 != 0);

    let k_hook = SetWindowsHookExA(
      WH_MOUSE_LL,
      Some(m_callback),
      HINSTANCE::default(),
      0
    );

    let mut message = MSG::default();

    while GetMessageA(&mut message, HWND::default(), 0, 0).into() {
      DispatchMessageA(&message);
    }

    if k_hook.is_err() {
      UnhookWindowsHookEx(k_hook.unwrap());
    }

    Ok(())
  }
}

extern "system" fn m_callback(
  ncode: i32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  unsafe {
    if wparam.0 as u32 == WM_LBUTTONUP && ncode as u32 == HC_ACTION {
      let pma_coordinate = *(lparam.0 as *const u16);
      dbg!(pma_coordinate);
  
      let builder =
        thread::Builder::new().name("Click counter".into());
      let counter = Arc::clone(&COUNT);
  
      let handle = builder.spawn(move || {
        if let LockResult::Ok(value) = counter.lock() {
          value.fetch_add(1, Ordering::SeqCst);
          let text = format!("Click number: {:?} \n", value);
          let stdout = io::stdout();
          let mut handle = stdout.lock();
          handle.write_all(text.as_bytes()).unwrap();
        }
        let _ = io::stdout().flush();
        assert_eq!(thread::current().name(), Some("Click counter"));
      });
  
      handle.unwrap().join().unwrap();
    }

    CallNextHookEx(HHOOK::default(), ncode, wparam, lparam)
  }
}
