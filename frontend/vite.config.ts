import { defineConfig } from "vite";
import { qwikVite } from "@builder.io/qwik/optimizer";
import { qwikCity } from "@builder.io/qwik-city/vite";
import { resolve } from "path";

export default defineConfig(() => {
  return {
    plugins: [qwikCity({ platform: { checkOrigin: false } }), qwikVite()],
    resolve: {
      alias: {
        "~": resolve(__dirname, "src"),
      },
    },
    server: {
      port: 5173,
      proxy: {
        "/api": {
          target: "http://localhost:3001",
          changeOrigin: true,
        },
      },
    },
    preview: {
      port: 4173,
    },
  };
});
