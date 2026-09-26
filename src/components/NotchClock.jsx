import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import "./NotchClock.css";

export default function NotchClock() {
  const [now, setNow] = useState(() => new Date());
  const [visible, setVisible] = useState(true);

  // Update clock every second
  useEffect(() => {
    const id = setInterval(() => {
      setNow(new Date());
    }, 1000);

    return () => clearInterval(id);
  }, []);

  // Listen for mouse-triggered notch events from Rust
  useEffect(() => {
    let unlistenShow;
    let unlistenHide;

    const setupListeners = async () => {
      unlistenShow = await listen("notch-show", () => {
        console.log("SHOW NOTCH");
        setVisible(true);
      });

      unlistenHide = await listen("notch-hide", () => {
        console.log("HIDE NOTCH");
        setVisible(false);
      });
    };

    setupListeners();

    return () => {
      if (unlistenShow) {
        unlistenShow();
      }

      if (unlistenHide) {
        unlistenHide();
      }
    };
  }, []);

  const hours12 = now.getHours() % 12 || 12;
  const hh = String(hours12).padStart(2, "0");
  const mm = String(now.getMinutes()).padStart(2, "0");

  const dayNum = now.getDate();
  const weekday = now.toLocaleDateString("en-US", {
    weekday: "short",
  });

  const month = now.toLocaleDateString("en-US", {
    month: "short",
  });

  return (
    <div
      className={`notch-clock ${visible ? "notch-visible" : "notch-hidden"}`}
      role="timer"
      aria-live="off"
      aria-label={`${hh}:${mm} ${weekday}, ${month} ${dayNum}`}
    >
      <div className="time-container">
        <div className="hr-container">
          {hh}
        </div>

        <div className="min-container">
          {mm}
        </div>
      </div>

      <div className="date-container">
        <span className="date-day">
          {String(dayNum).padStart(2, "0")}
        </span>

        <span className="date-rest">
          {weekday}, {month}
        </span>
      </div>
    </div>
  );
}