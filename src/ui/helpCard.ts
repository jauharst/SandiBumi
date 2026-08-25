// The module Help card (docs/guidebook_prompt.md): summary, THE EQUATIONS, the
// published references, and the link into the module's guidebook chapter. ONE
// builder used by both help entry points — the pane's own "? Help" button and the
// workspace's "Help for this panel…" — so the two can never show different content.
import { getModuleHelp, moduleGuideStatus, openModuleGuide } from "../ipc";
import { setStatus } from "../state";

/** Render the help content for one module. `fallbackDoc` is the manifest doc,
 *  shown whole when the module has no card yet. */
export async function buildModuleHelpCard(name: string, fallbackDoc?: string): Promise<HTMLElement> {
  const content = document.createElement("div");
  const para = (cls: string, text: string) => {
    const p = document.createElement("p");
    p.className = cls;
    p.textContent = text;
    content.appendChild(p);
  };
  const help = await getModuleHelp(name).catch(() => null);
  if (help) {
    para("help-body", help.summary);
    const eq = document.createElement("pre");
    eq.className = "help-equations";
    eq.textContent = help.equations.join("\n");
    content.appendChild(eq);
    if (help.references.length) {
      const refs = document.createElement("div");
      refs.className = "help-refs";
      for (const r of help.references) {
        const line = document.createElement("div");
        line.textContent = r;
        refs.appendChild(line);
      }
      content.appendChild(refs);
    }
    if (help.note) para("help-note", help.note);
  } else {
    para("help-body", fallbackDoc || "Documentation for this module is unavailable.");
  }
  if (await moduleGuideStatus(name).catch(() => false)) {
    const link = document.createElement("button");
    link.className = "btn help-guide-link";
    link.textContent = "See complete guidebook";
    link.addEventListener("click", () => {
      openModuleGuide(name).catch((e) => setStatus(String(e)));
    });
    content.appendChild(link);
  } else if (!help) {
    para("help-note", "Illustrated help for each panel will open here in a later release.");
  }
  return content;
}
