# Articles LXS integration guide

Articles owns markdown publication data and its optional independently-routed
SSR reader (`articles-ui`). An estate core links to `/articles`; it never
duplicates article CRUD or markdown rendering.

Compose the published `articles` API with a PostgreSQL grant and declare
public read routes (`/api/articles`, `/api/articles/slug/*`) separately from
authenticated authoring routes. Compose `articles-ui` for `/articles/*` when
the estate wants the portable reader surface. Markdown source should live in
the estate's `content/articles/` directory and be imported through
`tools/export-markdown.sh`, documented in `docs/export-markdown.md`, not
pasted manually into a database.

The export tool accepts an Auth bearer token at runtime only. Never commit a
token or use a public authoring route. Read `docs/api.md` and run the exporter
against a local estate before a release.
