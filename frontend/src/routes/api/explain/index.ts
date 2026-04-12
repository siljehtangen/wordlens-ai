import type { RequestHandler } from "@builder.io/qwik-city";

const BACKEND = process.env["BACKEND_URL"] ?? "http://localhost:3001";

export const onPost: RequestHandler = async ({ request, send }) => {
  const body = await request.text();

  let isStream = false;
  try {
    isStream = (JSON.parse(body) as { stream?: boolean }).stream === true;
  } catch {
    // ignore
  }

  let resp: Response;
  try {
    resp = await fetch(`${BACKEND}/api/explain`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body,
    });
  } catch {
    send(
      new Response(
        JSON.stringify({
          error:
            "Cannot reach the WordLens backend. Make sure `cargo run` is running on port 3001.",
        }),
        { status: 503, headers: { "Content-Type": "application/json" } },
      ),
    );
    return;
  }

  if (isStream) {
    send(
      new Response(resp.body, {
        status: resp.status,
        headers: {
          "Content-Type": "text/event-stream",
          "Cache-Control": "no-cache",
          "X-Accel-Buffering": "no",
        },
      }),
    );
  } else {
    const data = await resp.json();
    send(
      new Response(JSON.stringify(data), {
        status: resp.status,
        headers: { "Content-Type": "application/json" },
      }),
    );
  }
};
