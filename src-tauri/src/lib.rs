use device_query::{DeviceQuery, DeviceState};
use std::thread;
use std::time::Duration;

use tauri::{
    AppHandle,
    Emitter,
    Manager,
    PhysicalPosition,
};

// --------------------------------------------------
// Windows-specific imports
// --------------------------------------------------

#[cfg(target_os = "windows")]
use raw_window_handle::{
    HasWindowHandle,
    RawWindowHandle,
};

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowPos,
    HWND_NOTOPMOST,
    HWND_TOPMOST,
    SWP_NOACTIVATE,
    SWP_NOMOVE,
    SWP_NOSIZE,
};


// --------------------------------------------------
// Tauri entry point
// --------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let app_handle = app.handle().clone();

            start_mouse_tracker(app_handle);

            Ok(())
        })
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


// --------------------------------------------------
// Mouse tracker
// --------------------------------------------------

fn start_mouse_tracker(app_handle: AppHandle) {
  thread::spawn(move || {
      let device_state = DeviceState::new();

      // --------------------------------
      // Get primary monitor
      // --------------------------------

      let Some(window) =
          app_handle.get_webview_window("main")
      else {
          return;
      };

      let Ok(Some(monitor)) =
          window.primary_monitor()
      else {
          return;
      };

      let monitor_position = monitor.position();
      let monitor_size = monitor.size();

      let monitor_x = monitor_position.x;
      let monitor_y = monitor_position.y;

      let screen_width =
          monitor_size.width as i32;

      // --------------------------------
      // Trigger zone
      // --------------------------------

      let trigger_width = 400;
      let trigger_height = 5;

      let trigger_left =
          monitor_x
              + (screen_width - trigger_width) / 2;

      let trigger_right =
          trigger_left + trigger_width;

      // --------------------------------
      // Notch window
      // --------------------------------

      let notch_width = 300;
      let notch_height = 60;

      let notch_left =
          monitor_x
              + (screen_width - notch_width) / 2;

      let notch_right =
          notch_left + notch_width;

      // --------------------------------
      // Notch Y positions
      // --------------------------------

      let notch_visible_y = monitor_y;
      let notch_hidden_y = monitor_y - 50;

      // --------------------------------
      // State
      // --------------------------------

      let mut notch_visible = false;

      let mut hide_deadline:
          Option<std::time::Instant> = None;

      // --------------------------------
      // Main loop
      // --------------------------------

      loop {
          let mouse = device_state.get_mouse();

          let x = mouse.coords.0;
          let y = mouse.coords.1;

          // --------------------------------
          // Mouse inside trigger zone
          // --------------------------------

          let in_trigger =
              x >= trigger_left
                  && x <= trigger_right
                  && y >= monitor_y
                  && y <= monitor_y + trigger_height;

          // --------------------------------
          // Mouse inside actual notch
          // --------------------------------

          let in_notch =
              x >= notch_left
                  && x <= notch_right
                  && y >= monitor_y
                  && y <= monitor_y + notch_height;

          // --------------------------------
          // SHOW NOTCH
          // --------------------------------

          if !notch_visible && in_trigger {
              println!("🟢 SHOW NOTCH");

              if let Some(window) =
                  app_handle.get_webview_window("main")
              {
                  // Temporarily put notch above
                  // the foreground application.
                  //
                  // Does NOT activate the window.
                  set_window_topmost(
                      &window,
                      true,
                  );

                  // Animate:
                  // monitor_y - 50 → monitor_y
                  animate_window(
                      window,
                      notch_visible_y,
                  );
              }

              let _ =
                  app_handle.emit("notch-show", ());

              notch_visible = true;
              hide_deadline = None;
          }

          // --------------------------------
          // MOUSE IS INSIDE ACTIVE AREA
          // --------------------------------

          if notch_visible
              && (in_trigger || in_notch)
          {
              // Cancel pending hide.
              hide_deadline = None;
          }

          // --------------------------------
          // START HIDE TIMER
          // --------------------------------

          if notch_visible
              && !in_trigger
              && !in_notch
              && hide_deadline.is_none()
          {
              println!(
                  "🟡 Mouse left notch area"
              );

              hide_deadline = Some(
                  std::time::Instant::now()
                      + Duration::from_millis(300),
              );
          }

          // --------------------------------
          // HIDE AFTER DELAY
          // --------------------------------

          if let Some(deadline) =
              hide_deadline
          {
              if std::time::Instant::now()
                  >= deadline
              {
                  println!("🔴 HIDE NOTCH");

                  if let Some(window) =
                      app_handle.get_webview_window("main")
                  {
                      // Animate:
                      // monitor_y → monitor_y - 50
                      animate_window(
                          window.clone(),
                          notch_hidden_y,
                      );

                      // Remove TOPMOST after
                      // animation completes.
                      let window_for_z_order =
                          window.clone();

                      thread::spawn(move || {
                          thread::sleep(
                              Duration::from_millis(220),
                          );

                          set_window_topmost(
                              &window_for_z_order,
                              false,
                          );
                      });
                  }

                  let _ =
                      app_handle.emit("notch-hide", ());

                  notch_visible = false;
                  hide_deadline = None;
              }
          }

          // Check mouse every 50ms.
          thread::sleep(
              Duration::from_millis(50)
          );
      }
  });
}


fn animate_window(
  window: tauri::WebviewWindow,
  target_y: i32,
) {
  thread::spawn(move || {
      let Ok(start_position) =
          window.outer_position()
      else {
          return;
      };

      let start_y = start_position.y;

      let distance =
          target_y - start_y;

      // Already at target position.
      if distance == 0 {
          return;
      }

      // --------------------------------
      // Animation settings
      // --------------------------------

      let duration =
          Duration::from_millis(200);

      let frame_time =
          Duration::from_millis(10);

      let start_time =
          std::time::Instant::now();

      // --------------------------------
      // Animation loop
      // --------------------------------

      loop {
          let elapsed =
              start_time.elapsed();

          // --------------------------------
          // Animation finished
          // --------------------------------

          if elapsed >= duration {
              let _ =
                  window.set_position(
                      PhysicalPosition {
                          x: start_position.x,
                          y: target_y,
                      },
                  );

              break;
          }

          // --------------------------------
          // Progress: 0 → 1
          // --------------------------------

          let progress =
              elapsed.as_secs_f32()
                  / duration.as_secs_f32();

          // --------------------------------
          // Ease-out cubic
          //
          // Starts fast and slows down
          // near the final position.
          // --------------------------------

          let eased =
              1.0
                  - (1.0 - progress)
                      .powi(3);

          // --------------------------------
          // Calculate current Y
          // --------------------------------

          let current_y =
              start_y
                  + (distance as f32
                      * eased) as i32;

          // --------------------------------
          // Move window
          // --------------------------------

          let _ =
              window.set_position(
                  PhysicalPosition {
                      x: start_position.x,
                      y: current_y,
                  },
              );

          thread::sleep(frame_time);
      }
  });
}

// --------------------------------------------------
// Windows topmost control
// --------------------------------------------------

#[cfg(target_os = "windows")]
fn set_window_topmost(
    window: &tauri::WebviewWindow,
    topmost: bool,
) {
    let Ok(handle) = window.window_handle() else {
        return;
    };

    let RawWindowHandle::Win32(win32_handle) =
        handle.as_raw()
    else {
        return;
    };

    let hwnd = HWND(
        win32_handle.hwnd.get()
            as *mut std::ffi::c_void
    );

    unsafe {
        let insert_after = if topmost {
            HWND_TOPMOST
        } else {
            HWND_NOTOPMOST
        };

        let _ = SetWindowPos(
            hwnd,
            Some(insert_after),
            0,
            0,
            0,
            0,
            SWP_NOMOVE
                | SWP_NOSIZE
                | SWP_NOACTIVATE,
        );
    }
}


// --------------------------------------------------
// Non-Windows fallback
// --------------------------------------------------

#[cfg(not(target_os = "windows"))]
fn set_window_topmost(
    _window: &tauri::WebviewWindow,
    _topmost: bool,
) {
}


// --------------------------------------------------
// Window animation
// --------------------------------------------------

