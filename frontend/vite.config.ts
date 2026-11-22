import { svelte } from '@sveltejs/vite-plugin-svelte';
import { svelteTesting } from '@testing-library/svelte/vite';
import { defineConfig } from 'vitest/config';

// The gallery is a development tool: it never ships in the production bundle.
const entries = (mode: string) => ({
  main: 'index.html',
  ...(mode !== 'production' && { gallery: 'gallery.html' }),
});

export default defineConfig(({ mode }) => ({
  plugins: [svelte(), svelteTesting()],
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: {
    target: 'safari15',
    rollupOptions: { input: entries(mode) },
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./vitest.setup.ts'],
    include: ['src/**/*.test.ts', 'tools/**/*.test.ts'],
    exclude: ['tools/lint-fixtures/**', 'node_modules/**'],
  },
}));
