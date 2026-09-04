import { describe, expect, it } from "vitest";
import { validateReport } from "./report";

const report = {
  schema_version: 1,
  capture_id: "capture-12345678",
  captured_at: "2026-09-04T18:00:00.000Z",
  build: { revision: "abcdef123456" },
  surface: { role: "main", session_id: "session-12345678" },
};

describe("capability report validation", () => {
  it("accepts the report identity needed for storage", () => {
    expect(validateReport(report)).toEqual(report);
  });

  it("rejects unknown surface roles", () => {
    expect(validateReport({ ...report, surface: { ...report.surface, role: "top" } })).toBeNull();
  });

  it("rejects unsupported schemas", () => {
    expect(validateReport({ ...report, schema_version: 2 })).toBeNull();
  });
});
