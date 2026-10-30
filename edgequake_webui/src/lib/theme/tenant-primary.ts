/**
 * @module tenant-primary
 * @description Pure helpers for the tenant brand (primary) color.
 *
 * WHY pure: the runtime theme, the settings preview and the WCAG readout all call
 * these functions, so the numbers a tenant manager reviews are exactly the numbers
 * applied to every member's screen. No DOM, no React, no store.
 *
 * Dark-mode variant: the accent is re-lightened in OKLab (lightness only, hue and
 * chroma kept) until it reaches WCAG AA 4.5:1 on the dark page background.
 */

/** Default accent (`--primary` in `globals.css`, light theme). */
export const EDGEQUAKE_DEFAULT_PRIMARY = "#4b5bd6";

/** Page backgrounds and body ink the palette is checked against (`--background`, `--foreground`). */
export const LIGHT_PAGE_BG = "#fbfaf8";
export const LIGHT_PAGE_INK = "#1d1c1a";
export const DARK_PAGE_BG = "#1a1a19";
export const DARK_PAGE_INK = "#eceae5";

/** Text drawn on a solid accent: white, or the dark-theme button ink on a light accent. */
const WHITE = "#ffffff";
const DARK_THEME_ON_PRIMARY = "#12131c";

/** WCAG 2.2 AA minimum for normal text. */
export const WCAG_AA_TEXT = 4.5;

/**
 * Accent presets offered in the settings picker. Each passes AA text contrast on the
 * light page; dark mode is derived from them automatically.
 */
export const PRIMARY_PRESETS: readonly { hex: string; name: string }[] = [
  { hex: EDGEQUAKE_DEFAULT_PRIMARY, name: "Indigo (EdgeQuake default)" },
  { hex: "#2563eb", name: "Blue" },
  { hex: "#0f766e", name: "Teal" },
  { hex: "#15803d", name: "Green" },
  { hex: "#b45309", name: "Amber" },
  { hex: "#c2410c", name: "Orange" },
  { hex: "#be123c", name: "Rose" },
  { hex: "#6d28d9", name: "Violet" },
  { hex: "#475569", name: "Slate" },
];

export interface PrimaryPaletteMode {
  /** Solid accent: buttons, links, focus rings, active nav. */
  primary: string;
  /** Text drawn on top of `primary`. */
  foreground: string;
}

export interface PrimaryPalette {
  light: PrimaryPaletteMode;
  dark: PrimaryPaletteMode;
}

type Rgb = readonly [number, number, number];
interface Oklab {
  L: number;
  a: number;
  b: number;
}

/**
 * Parse `#rgb`, `#rrggbb` (the leading `#` is optional) into canonical `#rrggbb`.
 * Returns `null` for anything else, so the caller can show a validation message.
 */
export function normalizeHexColor(input: string | null | undefined): string | null {
  if (!input) return null;
  const match = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(input.trim());
  if (!match) return null;
  const digits = match[1].toLowerCase();
  const body = digits.length === 3 ? digits.replace(/./g, (c) => c + c) : digits;
  return `#${body}`;
}

function hexToRgb(hex: string): Rgb {
  const value = Number.parseInt(hex.slice(1), 16);
  return [(value >> 16) & 255, (value >> 8) & 255, value & 255];
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function rgbToHex([r, g, b]: readonly number[]): string {
  return `#${[r, g, b]
    .map((channel) => Math.round(clamp(channel, 0, 255)).toString(16).padStart(2, "0"))
    .join("")}`;
}

/** sRGB channel (0–255) → linear light (0–1). */
function toLinear(channel: number): number {
  const v = channel / 255;
  return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
}

/** Linear light (0–1, clamped) → sRGB channel (0–255). */
function fromLinear(linear: number): number {
  const v = clamp(linear, 0, 1);
  const encoded = v <= 0.0031308 ? 12.92 * v : 1.055 * v ** (1 / 2.4) - 0.055;
  return encoded * 255;
}

/** WCAG relative luminance of a `#rrggbb` color. */
export function relativeLuminance(hex: string): number {
  const [r, g, b] = hexToRgb(hex).map(toLinear);
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** WCAG contrast ratio between two `#rrggbb` colors, from 1 to 21. */
export function contrastRatio(a: string, b: string): number {
  const la = relativeLuminance(a);
  const lb = relativeLuminance(b);
  const [light, dark] = la >= lb ? [la, lb] : [lb, la];
  return (light + 0.05) / (dark + 0.05);
}

/** Pick whichever candidate reads better on `background`. */
export function readableForeground(background: string, light: string, dark: string): string {
  return contrastRatio(background, light) >= contrastRatio(background, dark) ? light : dark;
}

/** `#rrggbb` → OKLab (Björn Ottosson's matrices). */
function hexToOklab(hex: string): Oklab {
  const [r, g, b] = hexToRgb(hex).map(toLinear);
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  return {
    L: 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    a: 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    b: 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
  };
}

/** OKLab → `#rrggbb`, clipping out-of-gamut channels. */
function oklabToHex({ L, a, b }: Oklab): string {
  const l = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const m = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const s = (L - 0.0894841775 * a - 1.291485548 * b) ** 3;
  const r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
  const g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
  const bl = -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s;
  return rgbToHex([fromLinear(r), fromLinear(g), fromLinear(bl)]);
}

const LIGHTNESS_STEP = 0.01;
const MAX_LIGHTEN_STEPS = 100;

/**
 * Dark-mode accent: keep the hue and chroma, raise lightness just enough to reach
 * `minRatio` against the dark page. A color that already passes is returned unchanged.
 */
export function deriveDarkPrimary(hex: string, minRatio = WCAG_AA_TEXT): string {
  if (contrastRatio(hex, DARK_PAGE_BG) >= minRatio) return hex;
  const start = hexToOklab(hex);
  let candidate = hex;
  for (let step = 1; step <= MAX_LIGHTEN_STEPS; step += 1) {
    candidate = oklabToHex({ ...start, L: clamp(start.L + step * LIGHTNESS_STEP, 0, 1) });
    if (contrastRatio(candidate, DARK_PAGE_BG) >= minRatio) break;
  }
  return candidate;
}

/** Light + dark palette derived from one canonical accent color. */
export function derivePrimaryPalette(hex: string): PrimaryPalette {
  const dark = deriveDarkPrimary(hex);
  return {
    light: {
      primary: hex,
      foreground: readableForeground(hex, WHITE, LIGHT_PAGE_INK),
    },
    dark: {
      primary: dark,
      foreground: readableForeground(dark, WHITE, DARK_THEME_ON_PRIMARY),
    },
  };
}

/** The palette a page renders for `hex`, falling back to the EdgeQuake default. */
export function resolvePrimaryPalette(hex: string | null | undefined): PrimaryPalette {
  return derivePrimaryPalette(normalizeHexColor(hex) ?? EDGEQUAKE_DEFAULT_PRIMARY);
}

function modeDeclarations(mode: PrimaryPaletteMode): string {
  const declarations: ReadonlyArray<readonly [string, string]> = [
    ["--primary", mode.primary],
    ["--primary-foreground", mode.foreground],
    ["--ring", mode.primary],
    ["--sidebar-primary", mode.primary],
    ["--sidebar-primary-foreground", mode.foreground],
    ["--sidebar-ring", mode.primary],
  ];
  return declarations.map(([name, value]) => `${name}: ${value};`).join(" ");
}

/**
 * CSS that overrides the accent tokens for light (`:root`) and dark (`.dark`).
 * `.dark` comes after `:root` in the same sheet, so dark mode wins when both match.
 * Returns `""` for "no tenant color", which leaves the stylesheet defaults in place.
 */
export function buildPrimaryThemeCss(hex: string | null | undefined): string {
  const normalized = normalizeHexColor(hex);
  if (!normalized) return "";
  const palette = derivePrimaryPalette(normalized);
  return `:root { ${modeDeclarations(palette.light)} }\n.dark { ${modeDeclarations(palette.dark)} }\n`;
}
