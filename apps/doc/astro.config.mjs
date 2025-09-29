import { defineConfig } from 'astro/config';
import tailwind from '@astrojs/tailwind';
import react from '@astrojs/react';

export default defineConfig({
  integrations: [tailwind(), react()],
  output: 'static',
  site: process.env.GITHUB_PAGES === 'true' ? 'https://junkawasaki.github.io' : undefined,
  base: process.env.GITHUB_PAGES === 'true' ? '/jigen' : '/',
  build: {
    outDir: '../../docs',
  },
});
