// Calls to the editor's own server. Paths are relative, so the token in the page's path carries along.
// Every call answers { ok, status, data }; data is the parsed JSON, or the plain text of an error answer.
// A POST needs the JSON content type: the server refuses anything a cross-site form could send. It names the
// revision the page last saw in X-Courseplan-Revision, so the server can refuse a stale write.

async function answer(response) {
  const text = await response.text();
  let data = text;
  try {
    data = JSON.parse(text);
  } catch {
    // An error answer is plain text.
  }
  return { ok: response.ok, status: response.status, data };
}

// While the print preview is on, nothing may change the project: a write is refused here, whatever asked for it.
let readOnly = false;
export function setReadOnly(value) {
  readOnly = value;
}
const REFUSED = { ok: false, status: 423, data: "read-only" };

export async function get(path) {
  return answer(await fetch(path, { cache: "no-store" }));
}

export async function post(path, body = {}, revision = null) {
  if (readOnly) return REFUSED;
  const headers = { "Content-Type": "application/json" };
  if (revision) headers["X-Courseplan-Revision"] = revision;
  return answer(await fetch(path, { method: "POST", headers, body: JSON.stringify(body) }));
}

// A body that is not JSON: the answer is parsed like the others. `type` is the content type of the body; the server
// accepts only application/json and application/octet-stream.
export async function postBytes(path, bytes, revision = null) {
  if (readOnly) return REFUSED;
  const headers = { "Content-Type": "application/octet-stream" };
  if (revision) headers["X-Courseplan-Revision"] = revision;
  return answer(await fetch(path, { method: "POST", headers, body: bytes }));
}

// A file to hand to the person: { ok, status, blob, headers } on success, or the parsed error answer.
export async function getFile(path) {
  const response = await fetch(path, { cache: "no-store" });
  if (!response.ok) return answer(response);
  return { ok: true, status: response.status, blob: await response.blob(), headers: response.headers };
}
