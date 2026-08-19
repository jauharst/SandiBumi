/** Theme setting. The CSS in styles.css defines each palette as a `:root[data-theme="…"]`
 *  variable block (plus a `prefers-color-scheme` fallback for "system"); every component
 *  colour — including the dockview chrome via the --dv-* mapping — flows from those
 *  variables, so a theme only has to redefine them. This module manages the `data-theme`
 *  attribute and persistence.
 *
 *  Besides the built-in light/dark/system, four neutrally named colour themes ship as
 *  alternative light palettes (DEC-074, 2026-08-18: the former client-branded skins keep
 *  their palettes but lose the vendor names and any brand identity; all light +
 *  professional for print-adjacent work). */

export type ThemeChoice =
  | "light"
  | "dark"
  | "system"
  | "color-1"
  | "color-2"
  | "color-3"
  | "color-4"
  | "white";

/** Every theme except "system" is applied by setting `data-theme` to this value. */
export const THEMES: ThemeChoice[] = [
  "light",
  "dark",
  "system",
  "color-1",
  "color-2",
  "color-3",
  "color-4",
  "white",
];

/** DEC-074 (2026-08-18): stored ids from builds that shipped the client-branded theme
 *  names. Kept ONLY so an existing machine's preference still resolves to its palette
 *  after the rename; never offered anywhere, and the only place the old ids survive. */
const LEGACY_THEME_IDS: Record<string, ThemeChoice> = {
  pertamina: "color-1",
  halliburton: "color-2",
  schlumberger: "color-3",
  "lapi-itb": "color-4",
};

const STORAGE_KEY = "sandibumi.theme";

export function getTheme(): ThemeChoice {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored && stored in LEGACY_THEME_IDS) return LEGACY_THEME_IDS[stored];
  return stored && (THEMES as string[]).includes(stored) ? (stored as ThemeChoice) : "system";
}

export function setTheme(choice: ThemeChoice): void {
  localStorage.setItem(STORAGE_KEY, choice);
  applyTheme(choice);
}

export function applyStoredTheme(): void {
  applyTheme(getTheme());
}

function applyTheme(choice: ThemeChoice): void {
  const root = document.documentElement;
  if (choice === "system") {
    delete root.dataset.theme;
  } else {
    root.dataset.theme = choice;
  }
}
