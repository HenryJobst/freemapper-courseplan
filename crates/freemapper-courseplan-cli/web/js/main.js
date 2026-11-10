import { post } from "./api.js";
import { loadTexts, t } from "./texts.js";

const status = document.getElementById("status");

function say(id) {
  status.textContent = t(id);
}

async function closeEditor() {
  try {
    await post("api/quit");
    say("editor-status-closed");
  } catch {
    say("editor-status-lost");
  }
}

document.getElementById("menu-close").addEventListener("click", closeEditor);

try {
  await loadTexts();
  say("editor-status-ready");
} catch {
  // Without texts the page stays as it is; the server console says why.
}
