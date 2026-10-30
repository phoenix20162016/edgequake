import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const alias = {
  "@": path.resolve(__dirname, "./src"),
  "bun:test": path.resolve(__dirname, "./vitest.bun-shim.ts"),
};

export default defineConfig({
  resolve: { alias },
  test: {
    globals: true,
    projects: [
      {
        name: "unit-node",
        test: {
          environment: "node",
          include: ["src/**/*.test.ts"],
          exclude: ["src/**/*.test.tsx", "src/**/*.dom.test.ts"],
        },
        resolve: { alias },
      },
      {
        name: "unit-jsdom",
        test: {
          environment: "jsdom",
          include: ["src/**/*.test.tsx", "src/**/*.dom.test.ts"],
          setupFiles: ["./vitest.setup.ts"],
        },
        resolve: { alias },
      },
    ],
  },
});
