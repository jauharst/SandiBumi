/** Theme setting. The CSS in styles.css defines each palette as a `:root[data-theme="…"]`
 *  variable block (plus a `prefers-color-scheme` fallback for "system"); every component
 *  colour — including the dockview chrome via the --dv-* mapping — flows from those
 *  variables, so a theme only has to redefine them. This module manages the `data-theme`
 *  attribute and persistence.
 *
 *  Besides the built-in light/dark/system, there are client-branded skins used when the app
 *  is delivering analyses to a given operator/partner (colours matched to each brand's
 *  identity, all light + professional for print-adjacent work). */

export type ThemeChoice =
  | "light"
  | "dark"
  | "system"
  | "pertamina"
  | "halliburton"
  | "schlumberger"
  | "lapi-itb"
  | "white";

/** Every theme except "system" is applied by setting `data-theme` to this value. */
export const THEMES: ThemeChoice[] = [
  "light",
  "dark",
  "system",
  "pertamina",
  "halliburton",
  "schlumberger",
  "lapi-itb",
  "white",
];

const STORAGE_KEY = "sandibumi.theme";

export function getTheme(): ThemeChoice {
  const stored = localStorage.getItem(STORAGE_KEY);
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
