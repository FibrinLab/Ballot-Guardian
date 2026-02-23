(() => {
  const title = document.getElementById("hero-title");
  if (!title) return;

  const fullText = title.textContent || "";
  title.textContent = "";

  const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  if (reducedMotion) {
    title.textContent = fullText;
    return;
  }

  let i = 0;
  const tick = () => {
    title.textContent = fullText.slice(0, i);
    i += 1;
    if (i <= fullText.length) {
      window.setTimeout(tick, i < 10 ? 35 : 18);
    }
  };

  tick();
})();

