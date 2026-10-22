export type ThemePreference = "auto" | "light" | "dark";
export const THEME_KEY = "aiquota_theme";
export function readTheme(): ThemePreference {
  try {
    const saved = localStorage.getItem(THEME_KEY);
    return saved === "light" || saved === "dark" ? saved : "auto";
  } catch { return "auto"; }
}
export function applyTheme(preference: ThemePreference): () => void {
  const media = window.matchMedia("(prefers-color-scheme: dark)");
  const update = () => {
    const effective = preference === "auto" ? (media.matches ? "dark" : "light") : preference;
    document.documentElement.dataset.theme = effective;
    document.documentElement.style.colorScheme = effective;
  };
  update();
  media.addEventListener("change", update);
  return () => media.removeEventListener("change", update);
}
