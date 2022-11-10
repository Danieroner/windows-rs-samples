#![windows_subsystem = "windows"]

use windows::{
  core::*,
  UI::Notifications::{
    ToastNotificationManager, 
    ToastTemplateType, 
    ToastNotification, 
  },
};
use std::thread;
use std::time;

fn main() -> Result<()> {
  let builder = thread::Builder::new()
    .name(String::from("foo"))
    .stack_size(32* 1024 as usize);

  let duration = time::Duration::from_nanos(1);

  let handle = builder.spawn(move || -> Result<()> {
    let notification = {
      let toast_xml = 
        ToastNotificationManager::GetTemplateContent(ToastTemplateType::ToastText01)?;
  
      let text_node = toast_xml.GetElementsByTagName(&HSTRING::from("text"))?.Item(0)?;
      let text = toast_xml.CreateTextNode(&HSTRING::from("Example from Rust/WinRT"))?;
      text_node.AppendChild(&text)?;
  
      ToastNotification::CreateToastNotification(&toast_xml)?
    };

    ToastNotificationManager::GetDefault()?
      .CreateToastNotifierWithId(&HSTRING::from("{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe"))?
      .Show(&notification)?;

    thread::sleep(duration);
    assert_eq!(thread::current().name(), Some("foo"));

    Ok(())

  }).unwrap();

  handle.join().unwrap()?;

  Ok(())
}
