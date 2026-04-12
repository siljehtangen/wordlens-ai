import type { RequestHandler } from "@builder.io/qwik-city";

const BACKEND = "http://localhost:3001";

export const onPost: RequestHandler = async ({ request, json, env }) => {
  const body = await request.text();

  let resp: Response;
  try {
    resp = await fetch(`${BACKEND}/api/explain`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body,
    });
  } catch (err) {
    json(503, {
      error:
        "Cannot reach the WordLens backend. Make sure `cargo run` is running on port 3001.",
    });
    return;
  }

  const data = await resp.json();
  json(resp.status, data);
};
