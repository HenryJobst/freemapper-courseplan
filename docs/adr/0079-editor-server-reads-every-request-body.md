# 0079: The editor server reads every request body

Status: Accepted, 2026-11-12. Amends [0042](0042-editor-server-and-security.md).

## Context

`place_move_undo_over_http` in `tests/editor_server.rs` failed now and then on macOS CI with "Connection reset by
peer" from `read_to_string`. It never failed in 40 local runs.

The handlers of `api/undo`, `api/redo` and `api/quit` take no body, although the page and the test client send `{}`
with them. A handler that does not extract the body lets the server answer before the body has arrived. With
`Connection: close` the server then closes the connection with request bytes unread, and the kernel answers late bytes
with a reset. A reset after the full response makes the client's next read fail instead of returning the end of the
stream. I could not provoke it here, even with the body sent 200 ms after the head, so the chain above is the most
likely cause, not a proven one *(Claude's call)*.

## Decision

- The three handlers take the body as `Bytes` and drop it, so the server reads it before it answers. Every other POST
  handler already extracts it.
- The test client reads the answer with `read_answer`. It accepts a reset only after a complete response (head and as
  many body bytes as `Content-Length` says, or the final chunk). A reset earlier still fails the test.

## Cost if wrong

If the reset has another cause, the test client now hides a reset that arrives after a full response, which is
harmless for the test, while the server change costs nothing. The flake would then show up as a truncated response,
not as a reset.
