// Calls to the editor's own server. Paths are relative, so the token in the page's path carries along.
// A POST needs the content type below: the server refuses anything a cross-site form could send.

export async function get(path) {
  const response = await fetch(path, { cache: "no-store" });
  if (!response.ok) throw new Error(`${path}: ${response.status}`);
  return response.json();
}

export async function post(path, body = {}) {
  const response = await fetch(path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!response.ok) throw new Error(`${path}: ${response.status}`);
  return response.json();
}
