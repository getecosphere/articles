# articles-ui

Pluggable, native SSR UI for the `articles` domain. Compose this LXS beside
`articles` to publish a complete public reading surface without rebuilding
article pages in the host application.

- `/articles` — published article list with SEO/OpenGraph metadata.
- `/articles/:slug` — one published article rendered from its Markdown.

The binary calls the canonical Articles public API at request time. It owns
only reader-facing presentation; publishing, authorization and storage remain
in the `articles` backend.
