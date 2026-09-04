import { validateReport } from "./report";

interface Env {
  CAPABILITY_REPORTS: KVNamespace;
  REPORT_RATE_LIMITER: RateLimit;
}

const MAX_REPORT_BYTES = 256 * 1024;
const REPORT_TTL_SECONDS = 90 * 24 * 60 * 60;

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    if (url.pathname !== "/api/reports") {
      return json({ error: "Not found" }, 404);
    }
    return submitReport(request, env);
  },
} satisfies ExportedHandler<Env>;

async function submitReport(request: Request, env: Env): Promise<Response> {
  const requestError = validateRequest(request);
  if (requestError) {
    return requestError;
  }
  const rate = await env.REPORT_RATE_LIMITER.limit({ key: clientKey(request) });
  if (!rate.success) {
    return json({ error: "Too many reports. Try again in one minute." }, 429);
  }
  const body = await request.text();
  if (new TextEncoder().encode(body).byteLength > MAX_REPORT_BYTES) {
    return json({ error: "Report is too large" }, 413);
  }
  return persist(body, env);
}

async function persist(body: string, env: Env): Promise<Response> {
  const parsed = parseJson(body);
  const report = validateReport(parsed);
  if (!report) {
    return json({ error: "Invalid capability report" }, 422);
  }
  const receivedAt = new Date().toISOString();
  const receiptId = crypto.randomUUID().slice(0, 8);
  const key = `report:${receivedAt}:${receiptId}`;
  await env.CAPABILITY_REPORTS.put(key, body, {
    expirationTtl: REPORT_TTL_SECONDS,
    metadata: {
      build: report.build.revision,
      captureId: report.capture_id,
      receivedAt,
      role: report.surface.role,
      sessionId: report.surface.session_id,
    },
  });
  return json({ accepted_at: receivedAt, receipt_id: receiptId }, 201);
}

function validateRequest(request: Request): Response | null {
  if (request.method !== "POST") {
    return json({ error: "Method not allowed" }, 405, { Allow: "POST" });
  }
  if (request.headers.get("Origin") !== new URL(request.url).origin) {
    return json({ error: "Cross-origin reports are not accepted" }, 403);
  }
  const contentType = request.headers.get("Content-Type") ?? "";
  if (!contentType.startsWith("application/json")) {
    return json({ error: "Expected application/json" }, 415);
  }
  const length = Number(request.headers.get("Content-Length") ?? 0);
  return length > MAX_REPORT_BYTES ? json({ error: "Report is too large" }, 413) : null;
}

function clientKey(request: Request): string {
  return request.headers.get("CF-Connecting-IP") ?? "unknown-client";
}

function parseJson(body: string): unknown {
  try {
    return JSON.parse(body) as unknown;
  } catch {
    return null;
  }
}

function json(value: unknown, status: number, headers?: HeadersInit): Response {
  return Response.json(value, {
    status,
    headers: { "Cache-Control": "no-store", ...headers },
  });
}
