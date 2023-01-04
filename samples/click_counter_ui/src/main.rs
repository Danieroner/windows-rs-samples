#![windows_subsystem = "windows"]

use windows::{
  core::*,
  Win32::Foundation::*,
  Win32::System::LibraryLoader::GetModuleHandleA,
  Win32::UI::WindowsAndMessaging::*,
  Win32::Graphics::Gdi::*,
};
use once_cell::sync::Lazy;
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::LockResult;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static COUNT: Lazy<Arc<Mutex<AtomicUsize>>> =
  Lazy::new(|| Arc::new(Mutex::new(AtomicUsize::new(0))));
static HWND_S: Lazy<Arc<Mutex<HWND>>> =
  Lazy::new(|| Arc::new(Mutex::new(HWND::default())));

fn main() -> Result<()> {
  unsafe {
    let instance = GetModuleHandleA(None)?;
    debug_assert!(instance.0 != 0);
    let window_class = s!("Click Counter");
    let wc = WNDCLASSA   {
      lpszClassName: window_class,
      hInstance: instance,
      hbrBackground: GetSysColorBrush(COLOR_3DFACE),
      lpfnWndProc: Some(wndproc),
      hCursor: LoadCursorW(None, IDC_ARROW)?,
      ..Default::default()
    };
    let k_hook = SetWindowsHookExA(
      WH_MOUSE_LL,
      Some(m_callback),
      HINSTANCE::default(),
      0,
    );
    let atom = RegisterClassA(&wc);
    debug_assert!(atom != 0);

    CreateWindowExA(
      WINDOW_EX_STYLE::default(),
      window_class, s!("Click Counter"),
      WS_OVERLAPPEDWINDOW | WS_VISIBLE,
      100, 120, 350, 250,
      None,
      None,
      instance,
      None,
    );

    let mut message = MSG::default();

    if k_hook.is_err() {
      UnhookWindowsHookEx(k_hook.unwrap());
    }

    while GetMessageA(&mut message, HWND(0), 0, 0).into() {
      TranslateMessage(&message);
      DispatchMessageA(&message);
    }

    Ok(())
  }
}

fn create_label(window: HWND) {
  let counter = Arc::clone(&COUNT);
  if let LockResult::Ok(mut reference) = HWND_S.lock() {
    if let LockResult::Ok(value) = counter.lock() {
      let text = value.load(Ordering::Relaxed).to_string();
      let hstring = HSTRING::from(&text);
      let pcwstr = PCWSTR::from(&hstring);
      unsafe {
        *reference = CreateWindowExW(
          WINDOW_EX_STYLE::default(),
          w!("static"),
          pcwstr,
          WS_CHILD | WS_VISIBLE,
          160, 80, 200, 200, 
          window,
          HMENU(1),
          None,
          None,
        );
      }
    }
  }
}

extern "system" fn m_callback(
  ncode: i32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  unsafe {
    if wparam.0 as u32 == WM_LBUTTONUP && ncode as u32 == HC_ACTION {
      let builder =
        thread::Builder::new().name("Click counter".into());
      let counter = Arc::clone(&COUNT);
      let handle = builder.spawn(move || {
        if let LockResult::Ok(value) = counter.lock() {
          value.fetch_add(1, Ordering::SeqCst);
        }
        assert_eq!(thread::current().name(), Some("Click counter"));
      });
  
      handle.unwrap().join().unwrap();
  
      let counter = Arc::clone(&COUNT);
    
      if let LockResult::Ok(pointer) = HWND_S.lock() {
        if let LockResult::Ok(value) = counter.lock() {
          let text = value.load(Ordering::Relaxed).to_string();
          let hstring = HSTRING::from(text);
          SetWindowTextW(*pointer, PCWSTR::from(&hstring));
        }
      }
    }

    CallNextHookEx(HHOOK::default(), ncode, wparam, lparam)
  }
}

extern "system" fn wndproc(
  window: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  unsafe {
    match message as u32 {
      WM_PAINT => {
        create_label(window);
        let mut ps = PAINTSTRUCT::default();
        let hdc = BeginPaint(window, &mut ps);
        let hfont = CreateFontA(
          20, 0, 0, 0,
          FW_BOLD.0 as i32,
          0, 0, 0,
          SHIFTJIS_CHARSET.0 as u32,
          OUT_DEFAULT_PRECIS.0 as u32 ,
          CLIP_DEFAULT_PRECIS.0 as u32,
          DEFAULT_QUALITY.0 as u32,
          FF_ROMAN.0 as u32,
          None
        );
        SelectObject(hdc , hfont);
        TextOutA(hdc, 100, 50, s!("Click Number:").as_bytes());
        SelectObject(hdc , GetStockObject(SYSTEM_FONT));
        EndPaint(window, &ps);
        ReleaseDC(window, hdc);
      },
      WM_DESTROY => {
        DestroyWindow(window);
      },
      _ => {},
    }

    DefWindowProcA(window, message, wparam, lparam)
  }
}
