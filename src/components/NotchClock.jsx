import { useEffect, useState } from "react";
import "./NotchClock.css";



export default function NotchClock() {
  const [now, setNow] = useState(() => new Date());

  useEffect(() => {
    const id = setInterval(() => setNow(new Date()), 1000);
    return () => clearInterval(id);
  }, []);

  const hours12 = now.getHours() % 12 || 12;
  const hh = String(hours12).padStart(2, "0");
  const mm = String(now.getMinutes()).padStart(2, "0");

  const dayNum = now.getDate();
  const weekday = now.toLocaleDateString("en-US", { weekday: "short" });
  const month = now.toLocaleDateString("en-US", { month: "short" });

  return (
    <div className="notch-clock" role="timer" aria-live="off" aria-label={`${hh}:${mm} ${weekday}, ${month} ${dayNum}`}>
      <div className="time-container">

        <div className="hr-container">
          {hh}
        </div>
        <div className="min-container">
          {mm}
        </div>
       
      </div>
      <div className="date-container">
        <span className="date-day">{String(dayNum).padStart(2, "0")}</span>
        <span className="date-rest">
          {weekday}, {month}
        </span>
      </div>
    </div>
  );
}
