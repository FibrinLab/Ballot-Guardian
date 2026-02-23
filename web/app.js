(() => {
  const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  const typeText = (element, speed = 18) => {
    if (!element) return;
    const fullText = element.textContent || "";
    if (reducedMotion) return;

    element.dataset.typing = "true";
    element.textContent = "";

    let i = 0;
    const tick = () => {
      element.textContent = fullText.slice(0, i);
      i += 1;
      if (i <= fullText.length) {
        window.setTimeout(tick, i < 10 ? speed * 2 : speed);
      } else {
        element.dataset.typing = "false";
      }
    };

    tick();
  };

  typeText(document.getElementById("hero-title"), 15);

  const yearEl = document.getElementById("year");
  if (yearEl) yearEl.textContent = String(new Date().getFullYear());

  const votesSlider = document.getElementById("votes-slider");
  const repSlider = document.getElementById("rep-slider");
  const tokenSlider = document.getElementById("token-slider");

  const votesOutput = document.getElementById("votes-output");
  const costOutput = document.getElementById("cost-output");
  const marginalOutput = document.getElementById("marginal-output");
  const repOutput = document.getElementById("rep-output");
  const scaledOutput = document.getElementById("scaled-output");
  const displayOutput = document.getElementById("display-output");
  const tokenOutput = document.getElementById("token-output");
  const sqrtOutput = document.getElementById("sqrt-output");
  const adapterOutput = document.getElementById("adapter-output");

  const floorSqrt = (n) => Math.floor(Math.sqrt(n));

  const renderCalculators = () => {
    const votes = Number(votesSlider?.value || 10);
    const repBps = Number(repSlider?.value || 10_000);
    const tokens = Number(tokenSlider?.value || 400);

    const quadraticCost = votes * votes;
    const marginal = (votes + 1) * (votes + 1) - quadraticCost;
    const scaledTally = votes * repBps;
    const displayWeight = scaledTally / 10_000;

    const sqrtComponent = floorSqrt(tokens);
    const adapterWeight = Math.floor((sqrtComponent * repBps) / 10_000);

    if (votesOutput) votesOutput.textContent = String(votes);
    if (costOutput) costOutput.textContent = String(quadraticCost);
    if (marginalOutput) marginalOutput.textContent = String(marginal);

    if (repOutput) repOutput.textContent = `${(repBps / 10_000).toFixed(2)}x`;
    if (scaledOutput) scaledOutput.textContent = String(scaledTally);
    if (displayOutput) displayOutput.textContent = displayWeight.toFixed(2);

    if (tokenOutput) tokenOutput.textContent = String(tokens);
    if (sqrtOutput) sqrtOutput.textContent = String(sqrtComponent);
    if (adapterOutput) adapterOutput.textContent = String(adapterWeight);
  };

  [votesSlider, repSlider, tokenSlider].forEach((input) => {
    input?.addEventListener("input", renderCalculators);
  });

  renderCalculators();
})();
