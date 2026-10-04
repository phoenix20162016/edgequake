import { describe, expect, it } from "vitest";

import { buildPdfUploadFormData } from "../pdf-upload-form-data";
import { appendExtractionMode } from "../extraction-mode-field";

describe("appendExtractionMode (V05)", () => {
  it("writes the field only for an explicit mode", () => {
    const withMode = new FormData();
    appendExtractionMode(withMode, "decision");
    expect(withMode.get("extraction_mode")).toBe("decision");

    for (const choice of ["default", undefined, null] as const) {
      const untouched = new FormData();
      appendExtractionMode(untouched, choice);
      expect(untouched.has("extraction_mode")).toBe(false);
    }
  });

  it("keeps the PDF multipart body free of the field by default", () => {
    const pdf = new File(["x"], "a.pdf", { type: "application/pdf" });
    expect(buildPdfUploadFormData(pdf).has("extraction_mode")).toBe(false);
    expect(
      buildPdfUploadFormData(pdf, { extraction_mode: "llm" }).get("extraction_mode"),
    ).toBe("llm");
  });
});
