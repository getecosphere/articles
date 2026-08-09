# @articles/frontend

Pluggable Astro SSR frontend for the `articles` domain. Drop it into any
composition — no article code needed in the host.

```js
// astro.config.mjs (host composition)
import { defineConfig } from 'astro/config';
import articles from '@articles/frontend/integration';
export default defineConfig({
  integrations: [articles()],
});
```

That injects SSR routes:

- `/articles/` — published article list with SEO + OpenGraph
- `/articles/[slug]/` — one article, with per-article title/description/og:image

And exports reusable components: `<Articles/>`, `<ArticleCard/>`,
`<ArticleDetail/>`, `<ArticleLayout/>`.

The routes render inside the host layout if you pass `articles({ layout: Layout })`,
otherwise they use the domain's own public layout (stuff8 theme tokens).
