// articles-ui — standalone Astro SSR app for the articles domain.
// Serves /articles/ and /articles/[slug]/ with SEO + OpenGraph, backed by the
// articles REST API. Built as an adapter-node SSR app and bun-compiled into a
// single self-contained binary by `eco lxs build` (the astro-bun recipe).
import { defineConfig } from 'astro/config';
import node from '@astrojs/node';

export default defineConfig({
  output: 'server',
  adapter: node({ mode: 'standalone' }),
  site: process.env.PUBLIC_SITE_URL || 'http://localhost',
});
