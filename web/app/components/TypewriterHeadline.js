"use client";

import { useEffect, useState } from "react";

export default function TypewriterHeadline({ id, text, speed = 15 }) {
  const [displayText, setDisplayText] = useState(text);
  const [isTyping, setIsTyping] = useState(false);

  useEffect(() => {
    const media = window.matchMedia("(prefers-reduced-motion: reduce)");
    if (media.matches) {
      setDisplayText(text);
      setIsTyping(false);
      return;
    }

    let timeoutId;
    let cursor = 0;
    setDisplayText("");
    setIsTyping(true);

    const tick = () => {
      cursor += 1;
      setDisplayText(text.slice(0, cursor));

      if (cursor < text.length) {
        const delay = cursor < 10 ? speed * 2 : speed;
        timeoutId = window.setTimeout(tick, delay);
      } else {
        setIsTyping(false);
      }
    };

    timeoutId = window.setTimeout(tick, speed);
    return () => window.clearTimeout(timeoutId);
  }, [speed, text]);

  return (
    <h1 id={id} data-typing={isTyping ? "true" : "false"}>
      {displayText}
    </h1>
  );
}

