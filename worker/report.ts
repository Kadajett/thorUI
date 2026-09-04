export interface AcceptedReport {
  build: { revision: string };
  capture_id: string;
  captured_at: string;
  schema_version: number;
  surface: { role: string; session_id: string };
}

const CAPTURE_ID = /^[a-zA-Z0-9._-]{8,96}$/;
const ROLES = new Set(["main", "companion"]);

export function validateReport(value: unknown): AcceptedReport | null {
  if (!isRecord(value) || value.schema_version !== 1) {
    return null;
  }
  if (!validString(value.capture_id, CAPTURE_ID) || !validText(value.captured_at, 64)) {
    return null;
  }
  if (!isRecord(value.build) || !validText(value.build.revision, 80)) {
    return null;
  }
  if (!isRecord(value.surface) || !validSurface(value.surface)) {
    return null;
  }
  return value as unknown as AcceptedReport;
}

function validSurface(surface: Record<string, unknown>): boolean {
  return (
    typeof surface.role === "string" &&
    ROLES.has(surface.role) &&
    validText(surface.session_id, 96)
  );
}

function validString(value: unknown, pattern: RegExp): value is string {
  return typeof value === "string" && pattern.test(value);
}

function validText(value: unknown, maxLength: number): value is string {
  return typeof value === "string" && value.length > 0 && value.length <= maxLength;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
