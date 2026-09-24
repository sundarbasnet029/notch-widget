import { useEffect } from "react";
import { getCurrentWindow, primaryMonitor } from "@tauri-apps/api/window";
import { PhysicalPosition } from "@tauri-apps/api/dpi";
import { enable } from "@tauri-apps/plugin-autostart";

import NotchClock from "./components/NotchClock.jsx";
import "./App.css";

export default function App() {
  useEffect(() => {
    async function setupWindow() {
      const win = getCurrentWindow();

      const monitor = await primaryMonitor();
      if (!monitor) return;

      const size = await win.outerSize(); // PhysicalSize

      // Center horizontally on the primary monitor
      const x =
        monitor.position.x +
        Math.round((monitor.size.width - size.width) / 2);

      // Place at the very top of the monitor (notch position)
      // Use monitor.workArea.position.y if you prefer below the menu bar
      const y = monitor.position.y;

      await win.setPosition(new PhysicalPosition(x, y));

      // Explicitly ensure it is NOT always on top
      await win.setAlwaysOnTop(false);

      // If you started with "visible": false in tauri.conf.json
      await win.show();
    }

    async function setupAutostart() {
      try {
        await enable();
      } catch (err) {
        console.warn("Autostart enable failed:", err);
      }
    }

    setupWindow();
    setupAutostart();
  }, []);

  return <NotchClock />;
}