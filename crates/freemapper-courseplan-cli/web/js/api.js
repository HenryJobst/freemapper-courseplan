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

export async function get(path) {
  return answer(await fetch(path, { cache: "no-store" }));
}

export async function post(path, body = {}, revision = null) {
  const headers = { "Content-Type": "application/json" };
  if (revision) headers["X-Courseplan-Revision"] = revision;
  return answer(await fetch(path, { method: "POST", headers, body: JSON.stringify(body) }));
}
