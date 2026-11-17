// Browser lifecycle only. Time calculations and Canvas rendering live in Rust.
import init, { ClockApp } from "./pkg/rust_clock_view.js";

async function start() {
  await init();
  const clock = new ClockApp();
  const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
  let active = !document.hidden;
  let frameId = null;

  function animate(timestamp) {
    frameId = null;
    if (!active) return;
    clock.frame(timestamp, reducedMotion.matches);
    frameId = window.requestAnimationFrame(animate);
  }

  window.addEventListener("resize", () => clock.resize());

  document.addEventListener("visibilitychange", () => {
    if (document.hidden) {
      active = false;
      if (frameId !== null) window.cancelAnimationFrame(frameId);
      frameId = null;
      clock.pause();
    } else if (!active) {
      active = true;
      frameId = window.requestAnimationFrame(animate);
    }
  });

  if (active) frameId = window.requestAnimationFrame(animate);
}

start().catch((error) => {
  console.error("Rust Clock failed to start", error);
  const label = document.getElementById("digital-time");
  if (label) label.textContent = "読み込み失敗";
});
