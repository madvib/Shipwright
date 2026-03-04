import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwind from "@tailwindcss/vite";
import dts from "vite-plugin-dts";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));

export default defineConfig({
    plugins: [
        react(),
        tailwind(),
        dts({ include: ["src"], rollupTypes: true }),
    ],
    resolve: {
        alias: { "@": resolve(__dirname, "src") },
    },
    build: {
        lib: {
            entry: resolve(__dirname, "src/index.ts"),
            name: "ShipUI",
            formats: ["es"],
            fileName: "index",
        },
        rollupOptions: {
            // Don't bundle peers — consumers supply React
            external: ["react", "react-dom", "react/jsx-runtime"],
            output: {
                preserveModules: true,
                preserveModulesRoot: "src",
                globals: {
                    react: "React",
                    "react-dom": "ReactDOM",
                },
            },
        },
        cssCodeSplit: false,
        copyPublicDir: false,
    },
});
