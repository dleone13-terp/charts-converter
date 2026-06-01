import { defineConfig } from 'vite';

export default defineConfig({
  server: {
    port: 5173,
    proxy: {
      // Forward /tiles/* to tileserver-gl so the viewer dev server
      // and the tile server can run on different ports without CORS issues.
      '/tiles': {
        target: 'http://localhost:8080',
        changeOrigin: true,
        rewrite: path => path.replace(/^\/tiles/, ''),
      },
    },
  },
});
