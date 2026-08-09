/**
 * articlesIntegration — a drop-in Astro integration for the articles domain.
 *
 * Mount it in ANY composition's astro.config.mjs and the domain renders
 * itself: SSR routes for /articles/ and /articles/[slug]/ are injected into
 * the host app, with correct SEO + OpenGraph. The host does NOT need to
 * write any article code — this is the "use it without re-coding" contract.
 *
 * ```js
 * // astro.config.mjs
 * import { defineConfig } from 'astro/config';
 * import articles from '@articles/frontend/integration';
 * export default defineConfig({
 *   integrations: [articles()],
 * });
 * ```
 *
 * Optional options:
 * - `layout`: a component you want article pages to render inside (for
 *   example your estate Layout). If omitted, the domain uses its own
 *   public layout that matches the stuff8 theme.
 * - `apiUrl`: override the articles API base URL (defaults to
 *   PUBLIC_ARTICLES_URL env or /api/articles).
 */
import { fileURLToPath } from 'node:url';
import path from 'node:path';

/** @type {(opts?: { layout?: string; apiUrl?: string }) => import('astro').AstroIntegration} */
export default function articles(options = {}) {
  return {
    name: 'articles',
    hooks: {
      'astro:config:setup'({ config, injectRoute }) {
        if (options.layout) {
          const layoutAbs = path.resolve(fileURLToPath(config.root), options.layout);
          process.env.ARTICLES_HOST_LAYOUT = layoutAbs;
        }
        if (options.apiUrl) {
          process.env.ARTICLES_API_URL = options.apiUrl;
        }

        injectRoute({
          pattern: '/articles/',
          entryPoint: '@articles/frontend/src/routes/articles-index.astro',
        });
        injectRoute({
          pattern: '/articles/[slug]/',
          entryPoint: '@articles/frontend/src/routes/articles-detail.astro',
        });
      },
    },
  };
}
