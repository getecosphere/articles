# Markdown export

`tools/export-markdown.sh` creates a published Article through the Articles
LXS API from a Markdown source file. It is deliberately an exporter rather
than a direct database seeder: validation, slug generation, author identity,
and publish time remain owned by the Articles domain.

Source files use this small front-matter contract:

```md
---
title: A clear article title
excerpt: One concise summary.
tags: self-development, seven-habits
status: published
seo_title: Optional SEO title
seo_description: Optional SEO description
---

# Article body
```

Run it only against a running local/prod Articles API using a runtime bearer
token; do not commit tokens:

```bash
ARTICLES_TOKEN="$TOKEN" \
  lxs/articles/tools/export-markdown.sh content/articles/example.md \
  --api http://127.0.0.1:6233/api/articles
```

The token must satisfy the Articles author role. Re-running a document with
the same title will return a duplicate-slug conflict rather than silently
creating a second article.
