export type Skin = "classic" | "compact";
export const SKIN_KEY = "aiquota_skin";

/** Existing installations keep their original layout until explicitly switched. */
export function readSkin(): Skin {
  try {
    return localStorage.getItem(SKIN_KEY) === "compact" ? "compact" : "classic";
  } catch { return "classic"; }
}
export function applySkin(skin: Skin): void {
  document.documentElement.dataset.skin = skin;
}
