import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import "./NotchClock.css";

export default function NotchClock() {
  const [now, setNow] = useState(() => new Date());
  const [visible, setVisible] = useState(true);
  const [displayTime, setDisplayTime] = useState("00:00 AM");
  const [isScrambling, setIsScrambling] = useState(false);

  useEffect(() => {
    const id = setInterval(() => {
      setNow(new Date());
    }, 1000);

    return () => clearInterval(id);
  }, []);

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
      if (unlistenShow) unlistenShow();
      if (unlistenHide) unlistenHide();
    };
  }, []);

  const scrambleTo = (finalText) => {
    if (isScrambling) return;

    setIsScrambling(true);

    const chars = "0123456789";
    const duration = 300;
    const fps = 30;
    const totalFrames = Math.round((duration / 1000) * fps);
    let frame = 0;

    const interval = setInterval(() => {
      frame++;

      let result = "";
      for (let i = 0; i < finalText.length; i++) {
        const ch = finalText[i];
        if (ch === ":" || ch === " ") {
          result += ch;
        } else if (ch === "A" || ch === "P" || ch === "M") {
          // keep AM/PM letters stable during scramble
          result += ch;
        } else {
          result += chars[Math.floor(Math.random() * chars.length)];
        }
      }

      setDisplayTime(result);

      if (frame >= totalFrames) {
        clearInterval(interval);
        setDisplayTime(finalText);
        setIsScrambling(false);
      }
    }, 1000 / fps);
  };

  useEffect(() => {
    const hours12 = now.getHours() % 12 || 12;
    const hh = String(hours12).padStart(2, "0");
    const mm = String(now.getMinutes()).padStart(2, "0");
    const ampm = now.getHours() >= 12 ? "PM" : "AM";
    const newTime = `${hh}:${mm} ${ampm}`;

    if (newTime !== displayTime && !isScrambling) {
      const id = setTimeout(() => {
        scrambleTo(newTime);
      }, 0);

      return () => clearTimeout(id);
    }
  }, [now, displayTime, isScrambling]);

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
      aria-label={`${displayTime} ${weekday}, ${month} ${dayNum}`}
    >
      <div className="time-container">
        {displayTime}
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