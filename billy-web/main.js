import init, { Game, label } from "./pkg/billy_web.js";

const LOGICAL_W = 800;
const LOGICAL_H = 600;
const CELL_W = 16;
const CELL_H = 96;
const TICK_MS = 1000 / 60;
// The playfield is shifted down so the bottom platform is only half visible
// (the original 1920×1080 measurements give FIELD_Y ≈ 38 after /1.8 scaling).
const FIELD_Y = 38;
const HUD_Y = 3;

const FONT_STYLES = {
  white: [0x0006, 0x0046],
  lime: [0x004c, 0x008c],
  colorful: [0x0092, 0x009b],
  small: [0x009c, 0x00a5],
};
const fontByStyle = new Map();

const HOLE_ASSETS = {
  "1": "assets/hole/ALL16.DB-01b9.png",
  "2": "assets/hole/ALL16.DB-01ba.png",
  o: "assets/hole/ALL16.DB-01bb.png",
};

const HEART_FRAMES = ["a6", "a7", "a8", "a9", "aa", "ab"].map(
  (id) => `assets/heart/ALL16.DB-00${id}.png`,
);
const heartPhases = Array.from({ length: 12 }, () => Math.floor(Math.random() * 6));

const TIME_EMPTY = "assets/time/ALL16.DB-01a9.png";
const TIME_FILL = "assets/time/ALL16.DB-01aa.png";

const MAGIC_LADDER_FRAMES = [
  "bb", "bc", "bd", "be", "bf", "c0", "c1", "c2", "c3",
].map((id) => `assets/magic-ladder/ALL16.DB-00${id}.png`);

const BILLY_WALK = {
  src: "assets/billy-walk.png",
  frameW: 59,
  frameH: 66,
  frames: 12,
};
const BILLY_IDLE = "assets/billy-idle.png";
const BILLY_ANIM = {
  climb: ["00f0", "00f1", "00f3"],
  dig: ["0121", "0122", "0123", "0124"],
  drill: ["00f5", "00f6", "00f7", "00f8"],
  giggle: ["0113", "0114", "0115", "0116"],
  hammer: ["0119", "011a", "011b", "011c"],
  ouch: ["00fd", "00fe", "00ff", "0100"],
  throw: ["0129", "012a"],
};

function frameRange(start, end) {
  const ids = [];
  for (let id = start; id <= end; id += 1) {
    ids.push(id.toString(16).padStart(4, "0"));
  }
  return ids;
}

// First contiguous frame run in each Jungle monster directory = walk cycle.
const MONSTER_WALK_FRAMES = {
  banana: [
    "02bc", "02bd", "02c2", "02c3", "02be", "02c4",
    "02bf", "02c0", "02c5", "02c6", "02c1", "02c7",
  ],
  pumpkin: frameRange(0x02f9, 0x02fc),
  ananas: frameRange(0x02b7, 0x02b9),
  coconut: frameRange(0x031a, 0x031f),
  log: frameRange(0x0346, 0x0347),
  bi: frameRange(0x034a, 0x034f),
};
const MONSTER_IDLE = { banana: "02c8" };
const MONSTER_CLIMB = { banana: ["02da", "02db"] };
const MONSTER_HOLE = { banana: ["02d5", "02dc"] };
const MONSTER_HANDS = ["00ac", "00ad", "00ae"];

const INVENTORY_ICONS = {
  bomb: "assets/inventory/ALL16.DB-01ad.png",
  ladder: "assets/inventory/ALL16.DB-01ae.png",
  pickaxe: "assets/inventory/ALL16.DB-01b2.png",
  ball: "assets/inventory/ALL16.DB-01b0.png",
  glove: "assets/inventory/ALL16.DB-01b1.png",
  jackhammer: "assets/inventory/ALL16.DB-01b7.png",
  rainbow: "assets/inventory/ALL16.DB-01af.png",
  racket: "assets/inventory/ALL16.DB-01b5.png",
  boot: "assets/inventory/ALL16.DB-01b4.png",
  ring: "assets/inventory/ALL16.DB-01b6.png",
  permanent: "assets/inventory/ALL16.DB-01b8.png",
};

const canvas = document.getElementById("game");
const ctx = canvas.getContext("2d");
ctx.imageSmoothingEnabled = false;

// Platforms are composed on their own layer so the hole art can erase only
// the platform, leaving the painted background visible through the hole.
const platformCanvas = document.createElement("canvas");
platformCanvas.width = LOGICAL_W;
platformCanvas.height = LOGICAL_H;
const platformCtx = platformCanvas.getContext("2d");
platformCtx.imageSmoothingEnabled = false;

// Pixel art must be scaled by whole numbers; fractional CSS scaling is what
// makes an 800×600 canvas look soft.
function fitCanvas() {
  const dpr = Math.max(1, window.devicePixelRatio || 1);
  const availableW = Math.min(window.innerWidth - 40, 1600);
  const availableH = Math.max(300, window.innerHeight - 200);
  const scale = Math.max(
    1,
    Math.floor(Math.min(availableW / LOGICAL_W, availableH / LOGICAL_H)),
  );
  const cssW = LOGICAL_W * scale;
  const cssH = LOGICAL_H * scale;
  // Match the physical device pixels exactly so the browser never has to
  // interpolate the canvas bitmap itself.
  canvas.width = Math.round(cssW * dpr);
  canvas.height = Math.round(cssH * dpr);
  canvas.style.width = `${cssW}px`;
  canvas.style.height = `${cssH}px`;
  canvas.style.imageRendering = "pixelated";
  canvas.dataset.scale = String(scale);
  if (scaleEl) scaleEl.textContent = `${scale}× @${dpr.toFixed(2)}`;
  ctx.setTransform(
    canvas.width / LOGICAL_W,
    0,
    0,
    canvas.height / LOGICAL_H,
    0,
    0,
  );
  ctx.imageSmoothingEnabled = false;
}

const scoreEl = document.getElementById("score");
const timeEl = document.getElementById("time");
const livesEl = document.getElementById("lives");
const worldEl = document.getElementById("world");
const levelEl = document.getElementById("level");
const statusEl = document.getElementById("status");
const messageEl = document.getElementById("message");
const languageEl = document.getElementById("language");
const buildEl = document.getElementById("build");
const scaleEl = document.getElementById("scale");

const params = new URLSearchParams(location.search);
/// `?lab=hud` starts the HUD certification level instead of the demo.
const labMode = params.get("lab");
let locale = params.get("lang") || localStorage.getItem("open-billy-lang") || "en";
if (!["en", "da", "eo"].includes(locale)) locale = "en";

let game = null;
let projectConfig = null;
let running = true;
let lastFrame = performance.now();
let accumulator = 0;
const heldKeys = new Set();
const heldTouch = new Set();
const playerRender = {
  fromCol: null,
  fromRow: null,
  toCol: null,
  toRow: null,
  startTick: 0,
  duration: 1,
};
const entityRender = new Map();

const images = new Map();
function image(src) {
  let entry = images.get(src);
  if (!entry) {
    entry = new Image();
    entry.decoding = "async";
    entry.src = src;
    entry.failed = false;
    entry.addEventListener("error", () => {
      entry.failed = true;
    });
    images.set(src, entry);
  }
  return entry;
}

function ready(img) {
  return img && img.complete && img.naturalWidth > 0 && !img.failed;
}

function normalizeGlyphChar(value) {
  const first = String(value).split(" or ")[0].trim();
  if (!first) return "";
  return first.length === 1 ? first.toLowerCase() : first;
}

async function loadFont() {
  try {
    const response = await fetch("font-map.json", { cache: "no-store" });
    const data = await response.json();
    const chars = data.chars || {};
    const pending = [];
    for (const [style, range] of Object.entries(FONT_STYLES)) {
      const map = new Map();
      for (const [id, value] of Object.entries(chars)) {
        const code = parseInt(id, 16);
        if (code < range[0] || code > range[1]) continue;
        const ch = normalizeGlyphChar(value);
        if (ch.length !== 1 || map.has(ch)) continue;
        const entry = {
          img: image(`assets/font/ALL16.DB-${id}.png`),
          w: 8,
          h: 16,
        };
        map.set(ch, entry);
        pending.push(
          entry.img
            .decode()
            .then(() => {
              entry.w = entry.img.naturalWidth || entry.w;
              entry.h = entry.img.naturalHeight || entry.h;
            })
            .catch(() => {}),
        );
      }
      fontByStyle.set(style, map);
    }
    await Promise.all(pending);
  } catch {
    fontByStyle.clear();
  }
}

function lineHeightFor(style) {
  if (style === "colorful") return 26;
  if (style === "small") return 19;
  return 20;
}

function glyphFor(ch, style) {
  const direct = fontByStyle.get(style);
  if (direct && direct.has(ch)) return direct.get(ch);
  // White has no 0/1/2 in the atlas; use lime digits for any numeric run.
  if (/[0-9]/.test(ch)) {
    const lime = fontByStyle.get("lime");
    if (lime && lime.has(ch)) return lime.get(ch);
  }
  return null;
}

function drawBitmapText(text, x, yTop, style, scale, colors) {
  const lineH = lineHeightFor(style);
  let cursor = x;
  for (const raw of String(text)) {
    if (raw === " ") {
      cursor += 6 * scale;
      continue;
    }
    const glyph = glyphFor(raw.toLowerCase(), style);
    if (!glyph || !ready(glyph.img)) {
      ctx.save();
      ctx.font = `${Math.round(12 * scale)}px ui-monospace, monospace`;
      ctx.textBaseline = "alphabetic";
      ctx.fillStyle = colors?.text ?? "#e8f0f7";
      ctx.fillText(raw, cursor, yTop + (lineH - 4) * scale);
      cursor += ctx.measureText(raw).width + scale;
      ctx.restore();
      continue;
    }
    const w = glyph.img.naturalWidth;
    const h = glyph.img.naturalHeight;
    const gy = yTop + (lineH - h) * scale;
    ctx.drawImage(
      glyph.img,
      Math.round(cursor),
      Math.round(gy),
      Math.max(1, Math.round(w * scale)),
      Math.max(1, Math.round(h * scale)),
    );
    cursor += (w + 1) * scale;
  }
  return cursor;
}

/// The score is anchored at the right and drawn least-significant digit first.
/// The original advances every digit by a fixed 12 px, so wide digits (2, 4)
/// overlap their neighbour while narrow digits (1) leave a gap, and each digit
/// has its own small vertical offset — the deliberately "chaotic" look.
const SCORE_ADVANCE = 12;
const SCORE_Y_OFFSET = {
  0: -2,
  1: -4,
  2: 0,
  3: -3,
  4: 1,
  5: -3,
  6: -1,
  7: -3,
  8: 0,
  9: -2,
};

function drawScore(text, right, yTop, colors) {
  const chars = [...String(text)];
  const last = glyphFor(chars[chars.length - 1], "colorful");
  const lastWidth = last && ready(last.img) ? last.img.naturalWidth : 10;
  const start =
    right - (chars.length - 1) * SCORE_ADVANCE - lastWidth;

  // Right-to-left so a more significant digit is painted over the one to its
  // right, matching the original's layering.
  for (let i = chars.length - 1; i >= 0; i -= 1) {
    const ch = chars[i];
    const glyph = glyphFor(ch, "colorful");
    if (!glyph || !ready(glyph.img)) continue;
    const w = glyph.img.naturalWidth;
    const h = glyph.img.naturalHeight;
    const x = start + i * SCORE_ADVANCE;
    const y = yTop + (SCORE_Y_OFFSET[ch] ?? 0);
    ctx.drawImage(glyph.img, Math.round(x), Math.round(y), w, h);
  }
  return start;
}

/// Consumable counts use the small digit set.  Like the score they step by a
/// fixed advance and are drawn leftwards, so 2- and 3-digit counts overlap into
/// the space they need instead of being clipped or widened.
const COUNT_ADVANCE = 9;

function drawCount(text, lastDigitLeft, yTop, colors) {
  const chars = [...String(text)];
  let x = lastDigitLeft;
  for (let i = chars.length - 1; i >= 0; i -= 1) {
    const glyph = glyphFor(chars[i], "small");
    if (!glyph || !ready(glyph.img)) {
      x -= COUNT_ADVANCE;
      continue;
    }
    const w = glyph.img.naturalWidth;
    const h = glyph.img.naturalHeight;
    ctx.drawImage(
      glyph.img,
      Math.round(x),
      Math.round(yTop + (19 - h)),
      w,
      h,
    );
    x -= COUNT_ADVANCE;
  }
}

function measureBitmapText(text, style, scale) {  let width = 0;
  for (const raw of String(text)) {
    if (raw === " ") {
      width += 6 * scale;
      continue;
    }
    const glyph = glyphFor(raw.toLowerCase(), style);
    if (!glyph || !ready(glyph.img)) {
      width += 7 * scale;
    } else {
      width += (glyph.img.naturalWidth + 1) * scale;
    }
  }
  return width;
}

/// The same bitmap font, but as HTML images for the HUD/status card.
function bitmapTextHtml(text, style, scale) {
  const parts = [];
  for (const raw of String(text)) {
    if (raw === " ") {
      parts.push(`<span class="bitmap-space" style="width:${6 * scale}px"></span>`);
      continue;
    }
    const glyph = glyphFor(raw.toLowerCase(), style);
    if (!glyph) {
      parts.push(`<span class="bitmap-fallback">${escapeHtml(raw)}</span>`);
      continue;
    }
    const w = glyph.w || 8;
    const h = glyph.h || 16;
    parts.push(
      `<span class="bitmap-glyph">` +
        `<img src="${glyph.img.src}" alt="" style="width:${w * scale}px;height:${h * scale}px" ` +
        `onerror="this.style.display='none';this.nextElementSibling.style.display='inline'">` +
        `<span class="bitmap-fallback" style="display:none">${escapeHtml(raw)}</span>` +
        `</span>`,
    );
  }
  return `<span class="bitmap-text">${parts.join("")}</span>`;
}

// Used only when the wasm snapshot predates the config-driven fields.
const fallbackColors = {
  sky: "#0a1a12",
  platform: "#6b4a2a",
  platformEdge: "#b07c3e",
  ladder: "#d2aa5a",
  hole: "#000000b4",
  player: "#3d6fb6",
  playerSkin: "#e8c39e",
  playerHat: "#7a4a26",
  playerEye: "#151515",
  hazard: "#eb6446",
  collectible: "#f2c14e",
  portal: "#6ee39b",
  text: "#e8f0f7",
  shadow: "#0000005a",
  grid: "#ffffff09",
  panel: "#05090dbe",
};

function colorsOf(snapshot) {
  return snapshot.worldColors ?? fallbackColors;
}

function tileAssetsOf(snapshot) {
  return (
    snapshot.tileAssets ?? {
      platformOdd: `assets/${snapshot.world}/platform-a.png`,
      platformEven: `assets/${snapshot.world}/platform-b.png`,
      ladder: `assets/${snapshot.world}/ladder.png`,
      background: null,
      topAdornment: null,
    }
  );
}

function spritePath(token, snapshot) {
  return snapshot.tokenSprites?.[token] ?? `assets/sprites/${token}.png`;
}

function setMessage(text) {
  messageEl.textContent = text;
  if (!text) return;
  window.clearTimeout(setMessage.timer);
  setMessage.timer = window.setTimeout(() => {
    messageEl.textContent = "";
  }, 6000);
}

function applyLabels() {
  for (const el of document.querySelectorAll("[data-i18n]")) {
    const text = label(locale, el.dataset.i18n);
    el.innerHTML = bitmapTextHtml(text, "white", 1);
  }
  languageEl.value = locale;
}

function setLanguage(code) {
  locale = ["en", "da", "eo"].includes(code) ? code : "en";
  localStorage.setItem("open-billy-lang", locale);
  if (game) game.set_locale(locale);
  applyLabels();
  const url = new URL(location.href);
  url.searchParams.set("lang", locale);
  history.replaceState(null, "", url);
}

function inputBits() {
  let bits = 0;
  const down = (name) => heldKeys.has(name) || heldTouch.has(name);
  if (down("1")) bits |= 1;
  if (down("2")) bits |= 2;
  if (down("4")) bits |= 4;
  if (down("8")) bits |= 8;
  if (down("16")) bits |= 16;
  if (down("32")) bits |= 32;
  if (down("64")) bits |= 64;
  return bits;
}

function keyToBit(code) {
  switch (code) {
    case "ArrowLeft":
    case "KeyA":
      return "1";
    case "ArrowRight":
    case "KeyD":
      return "2";
    case "ArrowUp":
    case "KeyW":
      return "4";
    case "ArrowDown":
    case "KeyS":
      return "8";
    case "Space":
      return "16";
    case "ShiftLeft":
    case "ShiftRight":
      return "32";
    case "AltLeft":
    case "AltRight":
    case "KeyM":
      return "64";
    default:
      return null;
  }
}

window.addEventListener("keydown", (event) => {
  if (event.code === "KeyR") {
    event.preventDefault();
    restart();
    return;
  }
  const bit = keyToBit(event.code);
  if (bit) {
    event.preventDefault();
    heldKeys.add(bit);
  }
});

window.addEventListener("keyup", (event) => {
  const bit = keyToBit(event.code);
  if (bit) {
    event.preventDefault();
    heldKeys.delete(bit);
  }
});

window.addEventListener("blur", () => heldKeys.clear());

for (const button of document.querySelectorAll("#touch button")) {
  const bit = button.dataset.bit;
  const press = (event) => {
    event.preventDefault();
    heldTouch.add(bit);
    button.classList.add("active");
  };
  const release = (event) => {
    event.preventDefault();
    heldTouch.delete(bit);
    button.classList.remove("active");
  };
  button.addEventListener("pointerdown", press);
  button.addEventListener("pointerup", release);
  button.addEventListener("pointercancel", release);
  button.addEventListener("pointerleave", release);
  button.addEventListener("contextmenu", (event) => event.preventDefault());
}

languageEl.addEventListener("change", () => setLanguage(languageEl.value));

document.getElementById("restart").addEventListener("click", restart);

document.getElementById("level-file").addEventListener("change", async (event) => {
  const file = event.target.files?.[0];
  event.target.value = "";
  if (!file) return;
  try {
    const bytes = new Uint8Array(await file.arrayBuffer());
    const name = file.name.replace(/\.[^.]+$/, "");
    game = projectConfig
      ? Game.newWithConfigFiles(
          bytes,
          projectConfig.main,
          projectConfig.worldsJson,
          locale,
          name,
        )
      : new Game(bytes, locale, name);
    game.newRun(randomSeed());
    running = true;
    accumulator = 0;
    setMessage(`Loaded ${file.name}`);
  } catch (error) {
    setMessage(`Could not load ${file.name}: ${error}`);
  }
});

function randomSeed() {
  return (Math.random() * 0xffffffff) >>> 0;
}

function restart() {
  if (!game) return;
  try {
    game.newRun(randomSeed());
    running = true;
    accumulator = 0;
    setMessage("");
  } catch (error) {
    setMessage(`Restart failed: ${error}`);
  }
}

function frame(now) {
  const dt = Math.min(120, now - lastFrame);
  lastFrame = now;

  if (game) {
    if (running) {
      accumulator += dt;
      let steps = 0;
      while (accumulator >= TICK_MS && steps < 5) {
        game.step(inputBits());
        accumulator -= TICK_MS;
        steps += 1;
      }
      if (steps === 5) accumulator = 0;
    }
    const snapshot = JSON.parse(game.snapshot());
    draw(snapshot);
    if (snapshot.status !== "playing") running = false;
  }

  requestAnimationFrame(frame);
}

function draw(snapshot) {
  if (scoreEl) scoreEl.textContent = String(snapshot.score);
  if (timeEl) timeEl.textContent = String(Math.max(0, Math.ceil(snapshot.time)));
  if (livesEl) livesEl.textContent = String(snapshot.lives);
  worldEl.innerHTML = bitmapTextHtml(snapshot.worldLabel, "white", 1);
  levelEl.innerHTML = bitmapTextHtml(snapshot.level, "lime", 1);
  if (snapshot.build && buildEl) buildEl.textContent = snapshot.build;

  if (snapshot.status === "playing") {
    statusEl.hidden = true;
    statusEl.textContent = "";
  } else {
    statusEl.hidden = false;
    const title = bitmapTextHtml(snapshot.statusText, "white", 2);
    const controls = bitmapTextHtml(snapshot.strings.controls, "white", 1);
    statusEl.innerHTML = `<div class="status-card"><h2>${title}</h2><p>${controls}</p></div>`;
  }

  const colors = colorsOf(snapshot);
  const assets = tileAssetsOf(snapshot);
  const titleEl = document.querySelector(".brand h1");
  const subtitleEl = document.querySelector(".brand .subtitle");
  if (titleEl && snapshot.title) titleEl.textContent = snapshot.title;
  if (subtitleEl && snapshot.subtitle) subtitleEl.textContent = snapshot.subtitle;

  const background = assets.background ? image(assets.background) : null;
  if (background && ready(background)) {
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(background, 0, 0, LOGICAL_W, LOGICAL_H);
  } else {
    const gradient = ctx.createLinearGradient(0, 0, 0, LOGICAL_H);
    gradient.addColorStop(0, colors.sky);
    gradient.addColorStop(1, "#05080b");
    ctx.fillStyle = gradient;
    ctx.imageSmoothingEnabled = false;
    ctx.fillRect(0, 0, LOGICAL_W, LOGICAL_H);

    const glow = ctx.createRadialGradient(400, 240, 40, 400, 240, 460);
    glow.addColorStop(0, hexToRgba(colors.platformEdge, 0.18));
    glow.addColorStop(1, "rgba(0,0,0,0)");
    ctx.fillStyle = glow;
    ctx.imageSmoothingEnabled = false;
    ctx.fillRect(0, 0, LOGICAL_W, LOGICAL_H);
  }

  drawTiles(snapshot, colors, assets);
  drawEntities(snapshot, colors);
  drawPlayer(snapshot, colors);

  // World-specific top adornment (Jungle vines / Construction chains).
  const adornment = assets.topAdornment ? image(assets.topAdornment) : null;
  if (adornment && ready(adornment)) {
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(
      adornment,
      Math.round((LOGICAL_W - adornment.naturalWidth) / 2),
      0,
    );
  }

  drawGrid(colors);
  drawHud(snapshot, colors);
  if (snapshot.getReady && snapshot.status === "playing") drawReadyBanner(snapshot, colors);
}

function drawReadyBanner(snapshot, colors) {
  const text = snapshot.strings.getReady.toUpperCase();
  const scale = 2;
  const width = measureBitmapText(text, "white", scale);
  ctx.save();
  ctx.fillStyle = "rgba(0, 0, 0, 0.55)";
  ctx.fillRect(0, LOGICAL_H / 2 - 60, LOGICAL_W, 120);
  drawBitmapText(
    text,
    (LOGICAL_W - width) / 2,
    LOGICAL_H / 2 - lineHeightFor("white") * scale / 2,
    "white",
    scale,
    colors,
  );
  ctx.restore();
}

function drawTimeBar(snapshot, x, y, colors) {
  const empty = image(TIME_EMPTY);
  const fill = image(TIME_FILL);
  if (ready(empty) && ready(fill)) {
    const w = empty.naturalWidth;
    const h = empty.naturalHeight;
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(empty, x, y, w, h);
    const progress = Math.max(0, Math.min(1, snapshot.timeProgress ?? 0));
    ctx.save();
    ctx.beginPath();
    ctx.rect(x + 8, y + 3, (w - 16) * progress, h - 6);
    ctx.clip();
    ctx.drawImage(fill, x, y, w, h);
    ctx.restore();
  } else {
    const w = 141;
    ctx.fillStyle = colors.panel;
    ctx.fillRect(x, y, w, 16);
    ctx.fillStyle = colors.platformEdge;
    ctx.fillRect(x, y, w, 2);
    ctx.fillStyle = colors.lime ?? "#7bd88f";
    ctx.fillRect(x + 4, y + 3, (w - 8) * Math.max(0, Math.min(1, snapshot.timeProgress ?? 0)), 10);
  }
}

function drawInventoryIcon(name, x, y, centered = false) {
  const img = image(INVENTORY_ICONS[name]);
  if (!ready(img)) return { w: 20, h: 20 };
  const w = img.naturalWidth;
  const h = img.naturalHeight;
  // Permanents share the icon baseline; the short consumables are centred in
  // the same band so their counts sit right under them.  Tall icons such as
  // the jackhammer are clamped to the top edge instead of clipping.
  const top = centered ? y + 18 - Math.floor(h / 2) : Math.max(0, y + 33 - h);
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(img, x, top, w, h);
  return { w, h };
}

function drawInventory(snapshot, x, y, colors) {
  // Fixed slots in the original order, so a missing item leaves a gap instead
  // of shifting the later icons and score.  The three consumables advance by
  // their icon width (their counts need the room); the four permanents sit in
  // a tighter, evenly pitched group.
  const slots = [
    {
      icon: "ladder",
      advance: 48,
      centered: true,
      countShift: -4,
      show: (snapshot.magicLadders ?? 0) > 0,
      count: snapshot.magicLadders,
    },
    {
      icon: "ball",
      advance: 28,
      centered: true,
      countShift: -4,
      show: (snapshot.tennisBalls ?? 0) > 0,
      count: snapshot.tennisBalls,
    },
    {
      icon: "rainbow",
      advance: 45,
      centered: true,
      show: (snapshot.rainbow ?? 0) > 0,
      count: snapshot.rainbow,
    },
    { icon: "racket", advance: 35, show: Boolean(snapshot.racket), permanent: Boolean(snapshot.racketPermanent) },
    { icon: "glove", advance: 35, show: Boolean(snapshot.powerGlove), permanent: Boolean(snapshot.glovePermanent) },
    { icon: "boot", advance: 35, show: Boolean(snapshot.tennisShoes), permanent: Boolean(snapshot.shoesPermanent) },
    { icon: "ring", advance: 35, show: Boolean(snapshot.ring), permanent: Boolean(snapshot.ringPermanent) },
    {
      // One dig-tool slot: Bomb replaces Jackhammer replaces Pickaxe.
      icon:
        (snapshot.bombs ?? 0) > 0
          ? "bomb"
          : snapshot.jackhammer
            ? "jackhammer"
            : "pickaxe",
      advance: 35,
      countShift: -4,
      count: (snapshot.bombs ?? 0) > 0 ? snapshot.bombs : undefined,
      permanent:
        (snapshot.bombs ?? 0) === 0 &&
        Boolean(snapshot.jackhammer) &&
        Boolean(snapshot.jackhammerPermanent),
      show: true,
    },
  ];

  let cursor = x;
  for (const item of slots) {
    if (item.show) {
      const { w } = drawInventoryIcon(item.icon, cursor, y, item.centered);
      if (item.permanent) {
        const badge = image(INVENTORY_ICONS.permanent);
        if (ready(badge)) {
          ctx.drawImage(
            badge,
            cursor + Math.floor((w - badge.naturalWidth) / 2),
            y + 15,
          );
        }
      } else if (item.count !== undefined) {
        const text = String(item.count);
        const lastGlyph = glyphFor(text[text.length - 1], "small");
        const lastW =
          lastGlyph && ready(lastGlyph.img)
            ? lastGlyph.img.naturalWidth
            : COUNT_ADVANCE;
        // The whole number is centred under the icon; long counts grow to the
        // left with the fixed overlap, as in the original.
        const span = (text.length - 1) * COUNT_ADVANCE + lastW;
        const lastLeft =
          cursor +
          Math.floor((w - span) / 2) +
          3 +
          (item.countShift ?? 0) +
          (text.length - 1) * COUNT_ADVANCE;
        drawCount(text, lastLeft, y + 29, colors);
      }
    }
    cursor += item.advance;
  }
  return cursor;
}

function drawHud(snapshot, colors) {
  ctx.save();

  // Time is a progress bar at the far left; never a numeric countdown.
  drawTimeBar(snapshot, 8, HUD_Y + 2, colors);

  // Inventory in the original fixed slots: ladder, ball, rainbow, racket,
  // glove, boot, ring, bomb (then the jackhammer slot).
  drawInventory(snapshot, 160, HUD_Y, colors);

  // Lives on the right; each animated heart has its own random phase.  Above
  // ten lives the original shows one heart plus "xN" instead of more hearts.
  const HEART_SLOT = 19;
  const lives = Math.max(0, Number(snapshot.lives) || 0);
  const heartOverflow = lives > 10;
  const shown = heartOverflow ? 1 : lives;
  const heartLabel = heartOverflow ? `x${lives}` : "";
  const heartLabelWidth = heartOverflow
    ? measureBitmapText(heartLabel, "lime", 1)
    : 0;
  const heartsStart = Math.max(
    560,
    LOGICAL_W - 8 - shown * HEART_SLOT - (heartOverflow ? 2 + heartLabelWidth : 0),
  );

  // The score keeps a fixed right edge — collecting hearts never moves it —
  // and the gap to the ten-heart group is about one heart slot.
  drawScore(String(snapshot.score), 577, 9, colors);

  let hx = heartsStart;
  for (let i = 0; i < shown; i += 1) {
    const frame =
      (Math.floor(snapshot.tick / 6) + heartPhases[i]) % HEART_FRAMES.length;
    const heart = image(HEART_FRAMES[frame]);
    if (ready(heart)) {
      ctx.imageSmoothingEnabled = false;
      const hx2 = hx + Math.floor((HEART_SLOT - heart.naturalWidth) / 2);
      const hy = HUD_Y;
      ctx.drawImage(
        heart,
        hx2,
        hy,
        heart.naturalWidth,
        heart.naturalHeight,
      );
    }
    hx += HEART_SLOT;
  }
  if (heartOverflow) {
    drawBitmapText(heartLabel, hx + 2, 4, "lime", 1, colors);
  }
  ctx.restore();
}

function drawPlatform(context, col, row, colors, assets) {
  const x = col * CELL_W;
  const y = FIELD_Y + row * CELL_H;
  const src = row % 2 === 0 ? assets.platformOdd : assets.platformEven;
  const platform = src ? image(src) : null;
  if (platform && ready(platform) && platform.naturalWidth >= col * CELL_W + CELL_W) {
    context.imageSmoothingEnabled = false;
    context.drawImage(platform, col * CELL_W, 0, CELL_W, 32, x, y + CELL_H - 32, CELL_W, 32);
  } else {
    context.fillStyle = colors.platform;
    context.fillRect(x, y + CELL_H - 32, CELL_W, 32);
    context.fillStyle = colors.platformEdge;
    context.fillRect(x, y + CELL_H - 32, CELL_W, 5);
  }
}

function drawLadder(col, row, colors, assets) {
  const x = col * CELL_W;
  const y = FIELD_Y + row * CELL_H;
  const ladder = assets.ladder ? image(assets.ladder) : null;
  if (ladder && ready(ladder)) {
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(
      ladder,
      Math.round(x + CELL_W / 2 - ladder.naturalWidth / 2),
      y,
      ladder.naturalWidth,
      ladder.naturalHeight,
    );
  } else {
    ctx.strokeStyle = colors.ladder;
    ctx.lineWidth = 4;
    ctx.beginPath();
    ctx.moveTo(x + 5, y);
    ctx.lineTo(x + 5, y + CELL_H);
    ctx.moveTo(x + CELL_W - 5, y);
    ctx.lineTo(x + CELL_W - 5, y + CELL_H);
    for (let rung = 12; rung < CELL_H; rung += 16) {
      ctx.moveTo(x + 4, y + rung);
      ctx.lineTo(x + CELL_W - 4, y + rung);
    }
    ctx.stroke();
  }
}

function drawMagicLadder(col, row, snapshot, colors, assets) {
  const frame =
    MAGIC_LADDER_FRAMES[
      Math.floor(snapshot.tick / 6) % MAGIC_LADDER_FRAMES.length
    ];
  const art = image(frame);
  if (!ready(art)) {
    drawLadder(col, row, colors, assets);
    return;
  }
  const x = col * CELL_W + CELL_W / 2 - art.naturalWidth / 2;
  const y = FIELD_Y + row * CELL_H + CELL_H - art.naturalHeight;
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(
    art,
    Math.round(x),
    Math.round(y),
    art.naturalWidth,
    art.naturalHeight,
  );
}

function drawTiles(snapshot, colors, assets) {
  const holes = [];
  for (let row = 0; row < snapshot.grid.length; row += 1) {
    const line = snapshot.grid[row];
    for (let col = 0; col < line.length; col += 1) {
      const ch = line[col];
      if (ch === "1" || ch === "2" || ch === "o") {
        holes.push({ row, col, state: ch });
      }
    }
  }

  // 1. Compose every platform cell on a transparent layer.
  const pc = platformCtx;
  pc.clearRect(0, 0, LOGICAL_W, LOGICAL_H);
  pc.imageSmoothingEnabled = false;
  for (let row = 0; row < snapshot.grid.length; row += 1) {
    const line = snapshot.grid[row];
    for (let col = 0; col < line.length; col += 1) {
      const ch = line[col];
      if (
        ch === "#" ||
        ch === "L" ||
        ch === "N" ||
        ch === "1" ||
        ch === "2" ||
        ch === "o"
      ) {
        drawPlatform(pc, col, row, colors, assets);
      }
    }
  }

  // 2. Erase the platform with the original hole masks.  Only the alpha
  // channel matters; the dark red in the PNG is a mask colour, not art.
  for (const hole of holes) {
    const cx = hole.col * CELL_W + CELL_W / 2;
    const yTop = FIELD_Y + hole.row * CELL_H + CELL_H - 32;
    const worldArt =
      hole.state === "1"
        ? image(`assets/${snapshot.world}/hole-partly.png`)
        : hole.state === "2"
          ? image(`assets/${snapshot.world}/hole-mostly.png`)
          : null;
    const art =
      worldArt && ready(worldArt) ? worldArt : image(HOLE_ASSETS[hole.state]);
    pc.save();
    pc.globalCompositeOperation = "destination-out";
    if (ready(art)) {
      // Digging removes the top of the platform first; align the mask to the
      // walking surface, not the bottom of the 32px band.
      pc.drawImage(
        art,
        Math.round(cx - art.naturalWidth / 2),
        Math.round(yTop),
        art.naturalWidth,
        art.naturalHeight,
      );
    } else {
      const depth = hole.state === "o" ? 32 : hole.state === "2" ? 16 : 11;
      pc.beginPath();
      pc.ellipse(cx, yTop + depth / 2, CELL_W * 1.5, depth / 2, 0, 0, Math.PI * 2);
      pc.fill();
    }
    pc.restore();
  }

  // 3. The background is already on the main canvas; holes now show through.
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(platformCanvas, 0, 0, LOGICAL_W, LOGICAL_H);

  // 4. Ladders sit above the platform layer.
  for (let row = 0; row < snapshot.grid.length; row += 1) {
    const line = snapshot.grid[row];
    for (let col = 0; col < line.length; col += 1) {
      const ch = line[col];
      if (ch === "H" || ch === "L") {
        drawLadder(col, row, colors, assets);
      } else if (ch === "M" || ch === "N") {
        drawMagicLadder(col, row, snapshot, colors, assets);
      }
    }
  }
}

function drawFrameImage(img, sx, sy, sw, sh, centerX, bottomY, facing) {
  ctx.imageSmoothingEnabled = false;
  const dy = Math.round(bottomY - sh);
  if (facing === "left") {
    ctx.save();
    ctx.translate(Math.round(centerX), 0);
    ctx.scale(-1, 1);
    ctx.drawImage(img, sx, sy, sw, sh, -Math.round(sw / 2), dy, sw, sh);
    ctx.restore();
  } else {
    ctx.drawImage(
      img,
      sx,
      sy,
      sw,
      sh,
      Math.round(centerX - sw / 2),
      dy,
      sw,
      sh,
    );
  }
}

function drawMonster(entity, x, bottom, snapshot, t) {
  if (snapshot.world !== "jungle") return false;
  const phase = entity.phase || 0;
  const facing =
    entity.token === "banana"
      ? entity.facing === "left"
        ? "right"
        : "left"
      : "right";

  if (entity.trapped) {
    const holeFrames = MONSTER_HOLE[entity.token];
    if (holeFrames && holeFrames.length) {
      const index =
        (Math.floor(snapshot.tick / 5) + phase) % holeFrames.length;
      const img = image(
        `assets/monsters/${entity.token}/JUNGLE.DB-${holeFrames[index]}.png`,
      );
      if (!ready(img)) return false;
      const holeBottom =
        FIELD_Y + entity.row * CELL_H + CELL_H - 32 + 8;
      drawFrameImage(
        img,
        0,
        0,
        img.naturalWidth,
        img.naturalHeight,
        x,
        holeBottom,
        facing,
      );
      return true;
    }

    const row = snapshot.grid[entity.row] || "";
    const state = row[entity.col];
    const handIndex = state === "1" ? 0 : state === "2" ? 1 : 2;
    const img = image(
      `assets/monster-hands/ALL16.DB-${MONSTER_HANDS[handIndex]}.png`,
    );
    if (!ready(img)) return false;
    const holeBottom = FIELD_Y + entity.row * CELL_H + CELL_H - 32 + 8;
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(
      img,
      Math.round(x - img.naturalWidth / 2),
      Math.round(holeBottom - img.naturalHeight),
    );
    return true;
  }

  const climbFrames = MONSTER_CLIMB[entity.token];
  if (climbFrames && entity.moving && entity.dirY !== 0) {
    const index =
      Math.floor(Math.min(0.999, t) * climbFrames.length) %
      climbFrames.length;
    const img = image(
      `assets/monsters/${entity.token}/JUNGLE.DB-${climbFrames[index]}.png`,
    );
    if (ready(img)) {
      drawFrameImage(
        img,
        0,
        0,
        img.naturalWidth,
        img.naturalHeight,
        x,
        bottom,
        facing,
      );
      return true;
    }
  }

  if (!entity.moving) {
    const idle = MONSTER_IDLE[entity.token];
    if (idle) {
      const img = image(
        `assets/monsters/${entity.token}/JUNGLE.DB-${idle}.png`,
      );
      if (ready(img)) {
        drawFrameImage(
          img,
          0,
          0,
          img.naturalWidth,
          img.naturalHeight,
          x,
          bottom,
          facing,
        );
        return true;
      }
    }
  }

  const frames = MONSTER_WALK_FRAMES[entity.token];
  if (!frames || frames.length === 0) return false;
  const index =
    Math.floor(Math.min(0.999, t) * frames.length) % frames.length;
  const img = image(
    `assets/monsters/${entity.token}/JUNGLE.DB-${frames[index]}.png`,
  );
  if (!ready(img)) return false;
  drawFrameImage(
    img,
    0,
    0,
    img.naturalWidth,
    img.naturalHeight,
    x,
    bottom,
    facing,
  );
  return true;
}

function drawBillyAnim(name, tick, x, bottom, facing, rate) {
  const frames = BILLY_ANIM[name];
  if (!frames || frames.length === 0) return false;
  const index = Math.floor(tick / rate) % frames.length;
  const img = image(`assets/billy/${name}/JUNGLE.DB-${frames[index]}.png`);
  if (!ready(img)) return false;
  drawFrameImage(
    img,
    0,
    0,
    img.naturalWidth,
    img.naturalHeight,
    x,
    bottom,
    facing,
  );
  return true;
}

function drawBilly(snapshot, x, bottom) {
  const player = snapshot.player;
  if (player.work && drawBillyAnim(player.work, snapshot.tick, x, bottom, player.facing, 4)) {
    return true;
  }
  if (
    player.climbing &&
    drawBillyAnim(
      "climb",
      (player.climbProgress || 0) * 48,
      x,
      bottom,
      "right",
      4,
    )
  ) {
    return true;
  }
  if (
    snapshot.status === "complete" &&
    drawBillyAnim("giggle", snapshot.tick, x, bottom, "right", 6)
  ) {
    return true;
  }
  if (player.walking) {
    const sheet = image(BILLY_WALK.src);
    if (ready(sheet)) {
      const frame = Math.floor(snapshot.tick / 3) % BILLY_WALK.frames;
      drawFrameImage(
        sheet,
        frame * BILLY_WALK.frameW,
        0,
        BILLY_WALK.frameW,
        BILLY_WALK.frameH,
        x,
        bottom,
        player.facing,
      );
      return true;
    }
  }
  const idle = image(BILLY_IDLE);
  if (ready(idle)) {
    drawFrameImage(
      idle,
      0,
      0,
      idle.naturalWidth,
      idle.naturalHeight,
      x,
      bottom,
      player.facing,
    );
    return true;
  }
  return false;
}

function interpolatedEntity(entity, snapshot) {
  const duration = Math.max(1, snapshot.monsterMoveTicks || 30);
  let state = entityRender.get(entity.id);
  if (!state) {
    state = {
      fromCol: entity.col,
      fromRow: entity.row,
      toCol: entity.col,
      toRow: entity.row,
      startTick: snapshot.tick,
      duration,
      t: 0,
    };
    entityRender.set(entity.id, state);
  }

  if (entity.col !== state.toCol || entity.row !== state.toRow) {
    const done = Math.min(1, (snapshot.tick - state.startTick) / state.duration);
    const currentCol = state.fromCol + (state.toCol - state.fromCol) * done;
    const currentRow = state.fromRow + (state.toRow - state.fromRow) * done;
    const distance =
      Math.abs(entity.col - currentCol) + Math.abs(entity.row - currentRow);

    if (distance > 1.5) {
      state.fromCol = entity.col;
      state.fromRow = entity.row;
      state.toCol = entity.col;
      state.toRow = entity.row;
      state.startTick = snapshot.tick;
      state.duration = duration;
    } else {
      state.fromCol = currentCol;
      state.fromRow = currentRow;
      state.toCol = entity.col;
      state.toRow = entity.row;
      state.startTick = snapshot.tick;
      state.duration = duration;
    }
  }

  const t = Math.min(1, (snapshot.tick - state.startTick) / state.duration);
  state.t = t;
  return {
    col: state.fromCol + (state.toCol - state.fromCol) * t,
    row: state.fromRow + (state.toRow - state.fromRow) * t,
    t,
  };
}

function drawEntities(snapshot, colors) {
  for (const entity of snapshot.entities) {
    const position = interpolatedEntity(entity, snapshot);
    const x = position.col * CELL_W + CELL_W / 2;
    const bottom = FIELD_Y + position.row * CELL_H + CELL_H - 28;
    if (drawMonster(entity, x, bottom, snapshot, position.t)) continue;
    if (drawSprite(entity.token, x, bottom, snapshot)) continue;

    if (entity.kind === "portal") {
      ctx.fillStyle = colors.portal;
      ctx.beginPath();
      ctx.ellipse(x, bottom - 20, 11, 22, 0, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = colors.sky;
      ctx.beginPath();
      ctx.ellipse(x, bottom - 20, 6, 15, 0, 0, Math.PI * 2);
      ctx.fill();
    } else if (entity.kind === "hazard") {
      ctx.fillStyle = colors.hazard;
      ctx.beginPath();
      ctx.moveTo(x, bottom - 30);
      ctx.lineTo(x + 14, bottom);
      ctx.lineTo(x - 14, bottom);
      ctx.closePath();
      ctx.fill();
    } else {
      ctx.fillStyle = colors.collectible;
      ctx.beginPath();
      ctx.moveTo(x, bottom - 24);
      ctx.lineTo(x + 10, bottom - 12);
      ctx.lineTo(x, bottom);
      ctx.lineTo(x - 10, bottom - 12);
      ctx.closePath();
      ctx.fill();
      ctx.strokeStyle = hexToRgba(colors.platformEdge, 0.8);
      ctx.lineWidth = 2;
      ctx.stroke();
    }
  }
}

function interpolatedPlayer(snapshot) {
  const player = snapshot.player;
  const r = playerRender;
  if (player.climbing) {
    const row = player.renderRow ?? player.row;
    r.fromCol = player.col;
    r.fromRow = row;
    r.toCol = player.col;
    r.toRow = row;
    r.startTick = snapshot.tick;
    r.duration = 1;
    return { col: player.col, row };
  }
  if (r.toCol === null) {
    r.fromCol = player.col;
    r.fromRow = player.row;
    r.toCol = player.col;
    r.toRow = player.row;
    return { col: player.col, row: player.row };
  }

  if (player.col !== r.toCol || player.row !== r.toRow) {
    const done = Math.min(1, (snapshot.tick - r.startTick) / r.duration);
    const currentCol = r.fromCol + (r.toCol - r.fromCol) * done;
    const currentRow = r.fromRow + (r.toRow - r.fromRow) * done;
    const distance =
      Math.abs(player.col - currentCol) + Math.abs(player.row - currentRow);

    if (distance > 1.5) {
      r.fromCol = player.col;
      r.fromRow = player.row;
      r.toCol = player.col;
      r.toRow = player.row;
      r.startTick = snapshot.tick;
      r.duration = 1;
      return { col: player.col, row: player.row };
    }

    r.fromCol = currentCol;
    r.fromRow = currentRow;
    r.toCol = player.col;
    r.toRow = player.row;
    r.startTick = snapshot.tick;
    r.duration = player.climbing
      ? Math.max(1, snapshot.climbTicks || 8)
      : Math.max(1, snapshot.moveTicks || 4);
  }

  const t = Math.min(1, (snapshot.tick - r.startTick) / r.duration);
  return {
    col: r.fromCol + (r.toCol - r.fromCol) * t,
    row: r.fromRow + (r.toRow - r.fromRow) * t,
  };
}

function drawPlayer(snapshot, colors) {
  const position = interpolatedPlayer(snapshot);
  const x = position.col * CELL_W + CELL_W / 2;
  const bottom = FIELD_Y + position.row * CELL_H + CELL_H - 28;

  ctx.fillStyle = colors.shadow;
  ctx.beginPath();
  ctx.ellipse(x, bottom + 2, 11, 4, 0, 0, Math.PI * 2);
  ctx.fill();

  if (drawBilly(snapshot, x, bottom)) return;
  if (drawSprite("billy", x, bottom, snapshot)) return;

  const bob = Math.sin(snapshot.tick / 7) * 1.5;
  const top = bottom - 40 + bob;
  ctx.fillStyle = colors.playerSkin;
  ctx.fillRect(x - 6, top + 8, 12, 10);
  ctx.fillStyle = colors.playerHat;
  ctx.fillRect(x - 9, top + 2, 18, 7);
  ctx.fillStyle = colors.player;
  ctx.fillRect(x - 8, top + 18, 16, 15);
  ctx.fillStyle = shade(colors.player, 0.65);
  ctx.fillRect(x - 7, top + 33, 6, 7);
  ctx.fillRect(x + 1, top + 33, 6, 7);
  ctx.fillStyle = colors.playerEye;
  ctx.fillRect(x - 4, top + 12, 2, 2);
  ctx.fillRect(x + 2, top + 12, 2, 2);
}

function drawSprite(token, centerX, bottomY, snapshot) {
  const sprite = image(spritePath(token, snapshot));
  if (!ready(sprite)) return false;
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(
    sprite,
    Math.round(centerX - sprite.naturalWidth / 2),
    Math.round(bottomY - sprite.naturalHeight),
    sprite.naturalWidth,
    sprite.naturalHeight,
  );
  return true;
}

function drawGrid(colors) {
  ctx.strokeStyle = colors.grid;
  ctx.lineWidth = 1;
  for (let col = 0; col <= 50; col += 1) {
    ctx.beginPath();
    ctx.moveTo(col * CELL_W + 0.5, FIELD_Y);
    ctx.lineTo(col * CELL_W + 0.5, LOGICAL_H);
    ctx.stroke();
  }
  for (let row = 0; row <= 6; row += 1) {
    ctx.beginPath();
    ctx.moveTo(0, FIELD_Y + row * CELL_H + 0.5);
    ctx.lineTo(LOGICAL_W, FIELD_Y + row * CELL_H + 0.5);
    ctx.stroke();
  }
}

function hexToRgba(hex, alpha) {
  const clean = String(hex).replace("#", "");
  const r = parseInt(clean.slice(0, 2), 16) || 0;
  const g = parseInt(clean.slice(2, 4), 16) || 0;
  const b = parseInt(clean.slice(4, 6), 16) || 0;
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

function shade(hex, factor) {
  const clean = String(hex).replace("#", "");
  const r = Math.round((parseInt(clean.slice(0, 2), 16) || 0) * factor);
  const g = Math.round((parseInt(clean.slice(2, 4), 16) || 0) * factor);
  const b = Math.round((parseInt(clean.slice(4, 6), 16) || 0) * factor);
  return `rgb(${r}, ${g}, ${b})`;
}

function escapeHtml(text) {
  return String(text)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

async function loadProjectConfig() {
  for (const path of ["open-billy.toml", "config/open-billy.toml"]) {
    try {
      const response = await fetch(path, { cache: "no-cache" });
      if (!response.ok) continue;
      const main = await response.text();
      const match = main.match(/worlds\s*=\s*\[([^\]]*)\]/s);
      const paths = match
        ? [...match[1].matchAll(/"([^"]+)"/g)].map((m) => m[1])
        : [];
      const worlds = [];
      for (const worldPath of paths) {
        try {
          const worldResponse = await fetch(worldPath, { cache: "no-cache" });
          if (worldResponse.ok) {
            worlds.push([worldPath, await worldResponse.text()]);
          }
        } catch {
          // Skip a missing world file; the wasm loader will report it if needed.
        }
      }
      return { main, worldsJson: JSON.stringify(worlds) };
    } catch {
      // No config file beside the wasm build; use the embedded default.
    }
  }
  return null;
}

async function boot() {
  try {
    await init();
    await loadFont();
    projectConfig = await loadProjectConfig();
    game = labMode === "hud"
      ? projectConfig
        ? Game.demoHudWithConfigFiles(
            projectConfig.main,
            projectConfig.worldsJson,
            locale,
          )
        : Game.demoHud(locale)
      : projectConfig
        ? Game.demoWithConfigFiles(projectConfig.main, projectConfig.worldsJson, locale)
        : Game.demo(locale);
    game.newRun(randomSeed());
    applyLabels();
    if (params.get("debug") === "1" || labMode === "hud") {
      createDebugMenu();
    }
    requestAnimationFrame(frame);
  } catch (error) {
    setMessage(
      `Could not start the wasm build. Run ./build.sh first, then serve billy-web/dist. (${error})`,
    );
    console.error(error);
  }
}

/// Debug-only overlay for the browser shell.  It mutates the live simulation so
/// the HUD can be tested with any score, life count or inventory.
function createDebugMenu() {
  if (document.getElementById("debug-menu")) return;

  const style = document.createElement("style");
  style.textContent = `
    #debug-menu { position: fixed; right: 12px; bottom: 12px; z-index: 60;
      font: 12px/1.45 ui-monospace, monospace; color: #e8f0f7; }
    #debug-menu .toggle { float: right; background: #16334f; color: #e8f0f7;
      border: 1px solid #3d6fb6; border-radius: 4px; padding: 4px 10px;
      cursor: pointer; }
    #debug-body { clear: both; display: none; width: 236px; margin-top: 6px;
      background: rgba(8, 14, 22, 0.95); border: 1px solid #3d6fb6;
      border-radius: 6px; padding: 10px; box-shadow: 0 6px 24px #0008; }
    #debug-menu.open #debug-body { display: block; }
    #debug-body h3 { margin: 0 0 8px; font-size: 12px; letter-spacing: .08em;
      text-transform: uppercase; color: #9fc4ff; }
    #debug-body label { display: flex; align-items: center; justify-content: space-between;
      gap: 8px; margin: 2px 0; }
    #debug-body input[type=number] { width: 76px; background: #0c1a28;
      color: #e8f0f7; border: 1px solid #2b4b7a; border-radius: 3px; padding: 2px 4px; }
    #debug-body input[type=checkbox] { accent-color: #3d6fb6; }
    #debug-body fieldset { border: 1px solid #2b4b7a; border-radius: 4px;
      margin: 8px 0; padding: 4px 8px 6px; }
    #debug-body legend { color: #9fc4ff; padding: 0 4px; }
    #debug-body .apply { width: 100%; margin-top: 6px; background: #24507d;
      color: #fff; border: 1px solid #3d6fb6; border-radius: 4px;
      padding: 5px; cursor: pointer; }
  `;
  document.head.appendChild(style);

  const root = document.createElement("div");
  root.id = "debug-menu";
  root.innerHTML = `
    <button class="toggle" type="button">Debug</button>
    <div id="debug-body">
      <h3>HUD debug</h3>
      <label>Score <input id="dbg-score" type="number" min="0" step="1"></label>
      <label>Lives <input id="dbg-lives" type="number" min="0" max="999"></label>
      <label>Time (s) <input id="dbg-time" type="number" min="1" max="9999"></label>
      <fieldset>
        <legend>Items</legend>
        <label>Bombs <input id="dbg-bombs" type="number" min="0" max="999"></label>
        <label>Ladders <input id="dbg-ladders" type="number" min="0" max="999"></label>
        <label>Balls <input id="dbg-balls" type="number" min="0" max="999"></label>
        <label>Rainbow <input id="dbg-rainbow" type="number" min="0" max="999"></label>
        <label>Racket <input id="dbg-racket" type="checkbox"></label>
        <label>Racket P <input id="dbg-racket-perm" type="checkbox"></label>
        <label>Power glove <input id="dbg-glove" type="checkbox"></label>
        <label>Glove P <input id="dbg-glove-perm" type="checkbox"></label>
        <label>Tennis shoes <input id="dbg-shoes" type="checkbox"></label>
        <label>Shoes P <input id="dbg-shoes-perm" type="checkbox"></label>
        <label>Ring <input id="dbg-ring" type="checkbox"></label>
        <label>Ring P <input id="dbg-ring-perm" type="checkbox"></label>
        <label>Jackhammer <input id="dbg-jackhammer" type="checkbox"></label>
        <label>Jackhammer P <input id="dbg-jackhammer-perm" type="checkbox"></label>
      </fieldset>
      <button class="apply" type="button">Apply</button>
    </div>
  `;
  document.body.appendChild(root);

  const $ = (id) => root.querySelector(id);
  const snapshot = () => (game ? JSON.parse(game.snapshot()) : {});

  const syncFromGame = () => {
    const s = snapshot();
    $("#dbg-score").value = s.score ?? 0;
    $("#dbg-lives").value = s.lives ?? 3;
    $("#dbg-time").value = Math.max(1, Math.round(s.time ?? 0));
    $("#dbg-bombs").value = s.bombs ?? 0;
    $("#dbg-ladders").value = s.magicLadders ?? 0;
    $("#dbg-balls").value = s.tennisBalls ?? 0;
    $("#dbg-rainbow").value = s.rainbow ?? 0;
    $("#dbg-racket").checked = Boolean(s.racket);
    $("#dbg-glove").checked = Boolean(s.powerGlove);
    $("#dbg-shoes").checked = Boolean(s.tennisShoes);
    $("#dbg-ring").checked = Boolean(s.ring);
    $("#dbg-jackhammer").checked = Boolean(s.jackhammer);
    $("#dbg-racket-perm").checked = Boolean(s.racketPermanent);
    $("#dbg-glove-perm").checked = Boolean(s.glovePermanent);
    $("#dbg-shoes-perm").checked = Boolean(s.shoesPermanent);
    $("#dbg-ring-perm").checked = Boolean(s.ringPermanent);
    $("#dbg-jackhammer-perm").checked = Boolean(s.jackhammerPermanent);
  };

  const apply = () => {
    if (!game) return;
    const num = (id) => Math.max(0, Number($(id).value) || 0);
    game.debugSetState(
      Math.min(4294967295, num("#dbg-score")),
      Math.min(999, num("#dbg-lives")),
      Math.max(1, num("#dbg-time")),
    );
    game.debugSetItems(
      Math.min(999, num("#dbg-bombs")),
      Math.min(999, num("#dbg-ladders")),
      Math.min(999, num("#dbg-balls")),
      Math.min(999, num("#dbg-rainbow")),
      $("#dbg-racket").checked,
      $("#dbg-shoes").checked,
      $("#dbg-ring").checked,
      $("#dbg-glove").checked,
      $("#dbg-jackhammer").checked,
      false,
    );
    game.debugSetPermanents(
      $("#dbg-racket-perm").checked,
      $("#dbg-shoes-perm").checked,
      $("#dbg-glove-perm").checked,
      $("#dbg-ring-perm").checked,
      $("#dbg-jackhammer-perm").checked,
    );
  };

  root.querySelector(".toggle").addEventListener("click", () => {
    root.classList.toggle("open");
    if (root.classList.contains("open")) syncFromGame();
  });
  root.querySelector(".apply").addEventListener("click", apply);
  for (const input of root.querySelectorAll("input")) {
    input.addEventListener("change", apply);
  }

  // Typing in the panel must not move Billy or restart the level.
  for (const type of ["keydown", "keyup"]) {
    root.addEventListener(type, (event) => event.stopPropagation());
  }
}

// Small debug hook for local feedback and automated smoke tests.
window.openBillyDebug = () => (game ? JSON.parse(game.snapshot()) : null);
window.openBillyRestart = restart;

fitCanvas();
window.addEventListener("resize", fitCanvas);
boot();
