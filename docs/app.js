// olive-rs Interactive Studio
document.addEventListener("DOMContentLoaded", () => {
  // Elements
  const canvas = document.getElementById("render-canvas");
  const ctx = canvas.getContext("2d");
  const btnPlay = document.getElementById("btn-play");
  const btnReset = document.getElementById("btn-reset");
  const timelineSlider = document.getElementById("timeline-slider");
  const frameDisplay = document.getElementById("frame-display");

  // Params
  const paramRot = document.getElementById("param-rot");
  const paramScale = document.getElementById("param-scale");
  const paramPosx = document.getElementById("param-posx");
  const paramCrop = document.getElementById("param-crop");
  const paramBlend = document.getElementById("param-blend");
  const paramOpac = document.getElementById("param-opac");

  const valRot = document.getElementById("val-rot");
  const valScale = document.getElementById("val-scale");
  const valPosx = document.getElementById("val-posx");
  const valCrop = document.getElementById("val-crop");
  const valOpac = document.getElementById("val-opac");

  // State
  let isPlaying = false;
  let currentFrame = 0;
  let animId = null;

  function formatTimecode(frames, fps = 30) {
    const hrs = Math.floor(frames / (fps * 3600));
    const mins = Math.floor((frames / (fps * 60)) % 60);
    const secs = Math.floor((frames / fps) % 60);
    const f = frames % fps;
    return `${String(hrs).padStart(2, "0")}:${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")};${String(f).padStart(2, "0")}`;
  }

  function renderComposite() {
    const w = canvas.width;
    const h = canvas.height;
    ctx.clearRect(0, 0, w, h);

    // 1. SolidGeneratorNode (Navy Background)
    ctx.fillStyle = "rgb(13, 26, 64)";
    ctx.fillRect(0, 0, w, h);

    // Grid lines for compositing studio
    ctx.strokeStyle = "rgba(255, 255, 255, 0.05)";
    ctx.lineWidth = 1;
    for (let x = 0; x < w; x += 40) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, h);
      ctx.stroke();
    }
    for (let y = 0; y < h; y += 40) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(w, y);
      ctx.stroke();
    }

    // Save before blend
    ctx.save();

    // Blend mode
    const blendMode = paramBlend.value;
    switch (blendMode) {
      case "multiply": ctx.globalCompositeOperation = "multiply"; break;
      case "screen": ctx.globalCompositeOperation = "screen"; break;
      case "overlay": ctx.globalCompositeOperation = "overlay"; break;
      case "add": ctx.globalCompositeOperation = "lighter"; break;
      case "difference": ctx.globalCompositeOperation = "difference"; break;
      default: ctx.globalCompositeOperation = "source-over"; break;
    }

    const opacity = parseFloat(paramOpac.value) / 100;
    ctx.globalAlpha = opacity;

    // Center transform
    const cx = w / 2;
    const cy = h / 2;
    const dx = parseFloat(paramPosx.value);
    const rotDeg = parseFloat(paramRot.value) + (currentFrame * 1.5);
    const scale = parseFloat(paramScale.value);
    const cropPct = parseFloat(paramCrop.value) / 100;

    ctx.translate(cx + dx, cy);
    ctx.rotate((rotDeg * Math.PI) / 180);
    ctx.scale(scale, scale);

    // Amber foreground box (200x120)
    const boxW = 220;
    const boxH = 140;
    const cropW = boxW * cropPct;

    // Apply crop clipping
    ctx.save();
    ctx.beginPath();
    ctx.rect(-boxW / 2 + cropW, -boxH / 2, boxW - cropW, boxH);
    ctx.clip();

    // Draw box gradient
    const grad = ctx.createLinearGradient(-boxW / 2, -boxH / 2, boxW / 2, boxH / 2);
    grad.addColorStop(0, "#f39c12");
    grad.addColorStop(1, "#e67e22");
    ctx.fillStyle = grad;
    ctx.fillRect(-boxW / 2, -boxH / 2, boxW, boxH);

    ctx.strokeStyle = "#ffffff";
    ctx.lineWidth = 3;
    ctx.strokeRect(-boxW / 2, -boxH / 2, boxW, boxH);

    // Label inside composite node
    ctx.fillStyle = "#ffffff";
    ctx.font = "bold 16px 'Outfit', sans-serif";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText("olive-rs DAG", 0, -10);

    ctx.font = "11px 'JetBrains Mono', monospace";
    ctx.fillText(`Frame: ${currentFrame} | Rot: ${Math.round(rotDeg % 360)}°`, 0, 15);

    ctx.restore();
    ctx.restore();

    // Canvas Overlay HUD
    ctx.fillStyle = "rgba(0, 0, 0, 0.6)";
    ctx.fillRect(10, 10, 200, 50);
    ctx.strokeStyle = "#2b3240";
    ctx.strokeRect(10, 10, 200, 50);

    ctx.fillStyle = "#8db600";
    ctx.font = "11px 'JetBrains Mono', monospace";
    ctx.textAlign = "left";
    ctx.fillText(`TC: ${formatTimecode(currentFrame)}`, 20, 28);
    ctx.fillStyle = "#8b949e";
    ctx.fillText(`Mode: ${blendMode.toUpperCase()} | Scale: ${scale.toFixed(2)}x`, 20, 46);
  }

  function updateControls() {
    valRot.textContent = `${paramRot.value}°`;
    valScale.textContent = `${paramScale.value}x`;
    valPosx.textContent = `${paramPosx.value} px`;
    valCrop.textContent = `${paramCrop.value}%`;
    valOpac.textContent = `${paramOpac.value}%`;
    frameDisplay.textContent = `Frame ${currentFrame} (${formatTimecode(currentFrame)})`;
    renderComposite();
  }

  // Event Listeners for Controls
  [paramRot, paramScale, paramPosx, paramCrop, paramBlend, paramOpac].forEach((input) => {
    input.addEventListener("input", updateControls);
  });

  timelineSlider.addEventListener("input", (e) => {
    currentFrame = parseInt(e.target.value, 10);
    updateControls();
  });

  btnPlay.addEventListener("click", () => {
    isPlaying = !isPlaying;
    btnPlay.textContent = isPlaying ? "⏸ Pause" : "▶ Play";
    if (isPlaying) {
      loop();
    } else {
      cancelAnimationFrame(animId);
    }
  });

  btnReset.addEventListener("click", () => {
    isPlaying = false;
    btnPlay.textContent = "▶ Play";
    cancelAnimationFrame(animId);
    currentFrame = 0;
    timelineSlider.value = 0;
    paramRot.value = 0;
    paramScale.value = 1.0;
    paramPosx.value = 0;
    paramCrop.value = 0;
    paramBlend.value = "normal";
    paramOpac.value = 90;
    updateControls();
  });

  function loop() {
    if (!isPlaying) return;
    currentFrame = (currentFrame + 1) % 121;
    timelineSlider.value = currentFrame;
    updateControls();
    setTimeout(() => {
      animId = requestAnimationFrame(loop);
    }, 1000 / 30);
  }

  // Initial render
  updateControls();

  // --- Interactive Timeline Section ---
  const trackV1 = document.getElementById("track-v1");
  const trackA1 = document.getElementById("track-a1");
  const historyItems = document.getElementById("history-items");
  const btnUndo = document.getElementById("btn-undo");
  const btnRedo = document.getElementById("btn-redo");

  let timelineState = [
    { type: "clip", name: "Clip A", len: 60, color: "block-clip" },
    { type: "clip", name: "Clip B", len: 60, color: "block-clip" },
    { type: "clip", name: "Clip C", len: 60, color: "block-clip" },
  ];

  let undoStack = [];
  let redoStack = [];

  function logAction(msg) {
    const item = document.createElement("div");
    item.className = "log-item active";
    item.textContent = `[${formatTimecode(currentFrame)}] ${msg}`;
    historyItems.prepend(item);
  }

  function renderTimeline() {
    trackV1.innerHTML = "";
    trackA1.innerHTML = "";

    timelineState.forEach((b) => {
      const el = document.createElement("div");
      el.className = `timeline-block ${b.color}`;
      el.style.width = `${b.len * 2}px`;
      el.innerHTML = `<span>${b.name}</span> <span class="block-len">${b.len}f</span>`;
      trackV1.appendChild(el);

      // Corresponding audio track
      const aEl = document.createElement("div");
      aEl.className = b.type === "gap" ? "timeline-block block-gap" : "timeline-block block-audio";
      aEl.style.width = `${b.len * 2}px`;
      aEl.innerHTML = `<span>${b.type === "gap" ? "Gap" : "Audio " + b.name.slice(-1)}</span> <span class="block-len">${b.len}f</span>`;
      trackA1.appendChild(aEl);
    });

    btnUndo.disabled = undoStack.length === 0;
    btnRedo.disabled = redoStack.length === 0;
  }

  function pushState(actionName) {
    undoStack.push(JSON.parse(JSON.stringify(timelineState)));
    redoStack = [];
    logAction(actionName);
  }

  document.getElementById("btn-trim-in").addEventListener("click", () => {
    pushState("BlockTrimCommand: Trim In Clip B (inserted 20f gap)");
    if (timelineState.length >= 2 && timelineState[1].type === "clip") {
      timelineState[1].len = 40;
      timelineState.splice(1, 0, { type: "gap", name: "Gap", len: 20, color: "block-gap" });
    }
    renderTimeline();
  });

  document.getElementById("btn-trim-out").addEventListener("click", () => {
    pushState("BlockTrimCommand: Trim Out Clip B (shortened by 20f)");
    const target = timelineState.find(b => b.name === "Clip B");
    if (target) {
      target.len = Math.max(20, target.len - 20);
    }
    renderTimeline();
  });

  document.getElementById("btn-replace-gap").addEventListener("click", () => {
    pushState("TrackReplaceBlockWithGapCommand: Replaced Clip B with Gap");
    const idx = timelineState.findIndex(b => b.name === "Clip B");
    if (idx !== -1) {
      timelineState[idx] = { type: "gap", name: "Gap", len: timelineState[idx].len, color: "block-gap" };
    }
    renderTimeline();
  });

  document.getElementById("btn-insert-gap").addEventListener("click", () => {
    pushState("TrackListInsertGaps: Inserted 30f gap at start (time 0)");
    timelineState.unshift({ type: "gap", name: "Gap", len: 30, color: "block-gap" });
    renderTimeline();
  });

  btnUndo.addEventListener("click", () => {
    if (undoStack.length > 0) {
      redoStack.push(JSON.parse(JSON.stringify(timelineState)));
      timelineState = undoStack.pop();
      logAction("Undo previous timeline command");
      renderTimeline();
    }
  });

  btnRedo.addEventListener("click", () => {
    if (redoStack.length > 0) {
      undoStack.push(JSON.parse(JSON.stringify(timelineState)));
      timelineState = redoStack.pop();
      logAction("Redo timeline command");
      renderTimeline();
    }
  });

  renderTimeline();

  // --- Project JSON Inspector ---
  const sampleProject = {
    version: 230220,
    generator: "olive-rs 0.0.1",
    sequences: [
      {
        id: "a09819f2-51c3-4d6c-b3a1-2895e638ef90",
        name: "Main Timeline 4K",
        width: 3840,
        height: 2160,
        frame_rate_num: 60,
        frame_rate_den: 1,
        video_tracks: [
          {
            id: "e17a581e-bb36-4075-8025-06a928ef2311",
            name: "Video 1",
            track_type: "Video",
            blocks: [
              { id: "b10", kind: "Clip", name: "Title Card", length: { numer: 120, denom: 1 } },
              { id: "b11", kind: "Clip", name: "Drone Shot", length: { numer: 300, denom: 1 } }
            ]
          }
        ],
        audio_tracks: [
          {
            id: "d41a7741-2a14-411a-8f81-5d97bc161fa4",
            name: "Audio 1",
            track_type: "Audio",
            blocks: [
              { id: "b20", kind: "Clip", name: "Theme Music", length: { numer: 420, denom: 1 } }
            ]
          }
        ]
      }
    ],
    nodes: [
      { id: "n1", name: "Solid Generator", category: "generator", properties: { color: "#0d1a40" } },
      { id: "n2", name: "2D Transform", category: "distort", properties: { scale_x: "1.0", scale_y: "1.0" } },
      { id: "n3", name: "Merge Node", category: "math", properties: { blend_mode: "Normal", opacity: "1.0" } }
    ],
    connections: [
      { from_node: "n1", from_pin: "tex_out", to_node: "n2", to_pin: "tex_in" },
      { from_node: "n2", from_pin: "tex_out", to_node: "n3", to_pin: "blend_in" }
    ]
  };

  const jsonCode = document.getElementById("json-code");
  jsonCode.textContent = JSON.stringify(sampleProject, null, 2);

  document.getElementById("btn-copy-json").addEventListener("click", () => {
    navigator.clipboard.writeText(jsonCode.textContent).then(() => {
      const btn = document.getElementById("btn-copy-json");
      btn.textContent = "Copied!";
      setTimeout(() => (btn.textContent = "Copy JSON"), 2000);
    });
  });
});
