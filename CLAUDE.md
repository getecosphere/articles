# articles

Reusable domain that publishes and serves markdown articles with SSR-friendly
SEO/OpenGraph metadata. It owns article content and nothing else.

## What this domain owns

- Articles: title, slug, excerpt, markdown body, cover image, tags,
  draft/published status, author attribution, and SEO fields (seo_title,
  seo_description, og_image_url, published_at).
- Read API for published articles (used by SSR pages to render SEO + content).
- Write API for creating/editing/deleting articles.

## What this domain must NEVER own

- User accounts / identity — the auth domain owns identity and JWTs.
- CRM business data (customers, leads, partners, sales) — that is the CRM's
  monolith domain.
- In-app notifications — that is the notifications domain.

## Contracts (Public API)

- `GET /api/articles/health` — health check.
- `GET /api/articles` — list published articles (public, no auth).
- `GET /api/articles/slug/:slug` — one published article by slug (public).
- `GET /api/articles/all` — list all articles incl. drafts (admin/superadmin).
- `POST /api/articles` — create article (admin/superadmin only when
  `ARTICLES_ADMIN_ONLY=true`). Body:
  `{ title, content_md, excerpt?, cover_image_url?, tags?, status?, seo_title?, seo_description?, og_image_url? }`.
- `PUT /api/articles/:id` — update article (admin/superadmin).
- `DELETE /api/articles/:id` — delete article (admin/superadmin).

## Frontend

`frontend/` ships a pluggable Astro SSR package (`@articles/frontend`) that any
composition can mount with a single integration:

```js
// astro.config.mjs (host composition)
import articles from '@articles/frontend';
export default defineConfig({
  integrations: [articles({ layout: Layout, siteUrl: import.meta.env.PUBLIC_SITE_URL })],
});
```

It provides `<Articles/>` and `<ArticleDetail/>` components plus SSR routes
(`/articles/` and `/articles/[slug]/`) rendered inside the host layout with
correct SEO + OpenGraph. The composition does NOT need to re-code anything:
mounting the integration registers the routes and components.

## Runtime

`backend/` is Rust (Axum) + PostgreSQL 15. The service validates the estate's
shared HS512 JWT locally (same `JWT_SECRET` every domain uses) and grants write
access only to ADMIN/SUPERADMIN roles.

## Environment variables

- `JWT_SECRET` — shared signing key (required).
- `DATABASE_URL` — PostgreSQL connection string (Eco fills it).
- `SERVER_PORT` — listen port (Eco fills it).
- `CORS_ALLOWED_ORIGINS` — comma-separated browser origins.
- `PUBLIC_SITE_URL` — public origin used for canonical/OG URLs.
- `ARTICLES_ADMIN_ONLY` — default `true`; when true only admins may write.
