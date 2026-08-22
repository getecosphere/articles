#!/usr/bin/env bash
# Export a front-matter Markdown document into the Articles LXS API.
#
# Usage:
#   ARTICLES_TOKEN='<jwt>' tools/export-markdown.sh content.md --api http://127.0.0.1:PORT/api/articles
#
# The token is intentionally runtime-only. Never put it in a content file,
# shell history, ecompose.yml, or a committed .env.
set -euo pipefail

if [ "$#" -lt 1 ]; then
  echo "usage: ARTICLES_TOKEN='<jwt>' $0 <article.md> --api <articles-api-base>" >&2
  exit 64
fi

article_file=$1
shift
api_base=${ARTICLES_API_BASE:-}
while [ "$#" -gt 0 ]; do
  case "$1" in
    --api)
      api_base=${2:-}
      shift 2
      ;;
    *)
      echo "unknown option: $1" >&2
      exit 64
      ;;
  esac
done

if [ ! -f "$article_file" ]; then
  echo "article not found: $article_file" >&2
  exit 66
fi
if [ -z "${ARTICLES_TOKEN:-}" ]; then
  echo "ARTICLES_TOKEN is required (a bearer token with the Articles author role)." >&2
  exit 77
fi
if [ -z "$api_base" ]; then
  echo "--api or ARTICLES_API_BASE is required." >&2
  exit 64
fi
command -v curl >/dev/null || { echo "curl is required" >&2; exit 69; }
command -v jq >/dev/null || { echo "jq is required" >&2; exit 69; }

first_line=$(sed -n '1p' "$article_file")
if [ "$first_line" != '---' ]; then
  echo "article must begin with YAML-style front matter (---)." >&2
  exit 65
fi

front_matter=$(awk 'NR > 1 { if ($0 == "---") exit; print }' "$article_file")
content=$(awk 'NR == 1 { next } $0 == "---" { seen++; next } seen >= 1 { print }' "$article_file")
field() {
  printf '%s\n' "$front_matter" | awk -F': *' -v key="$1" '$1 == key { sub(/^[^:]*:[[:space:]]*/, ""); print; exit }'
}

title=$(field title)
excerpt=$(field excerpt)
tags=$(field tags)
status=$(field status)
seo_title=$(field seo_title)
seo_description=$(field seo_description)
if [ -z "$title" ] || [ -z "$content" ]; then
  echo "front matter needs title and the Markdown body cannot be empty." >&2
  exit 65
fi
status=${status:-published}

tags_json=$(printf '%s' "$tags" | jq -R 'split(",") | map(gsub("^\\s+|\\s+$"; "")) | map(select(length > 0))')
payload=$(jq -n \
  --arg title "$title" \
  --arg excerpt "$excerpt" \
  --arg contentMd "$content" \
  --arg status "$status" \
  --arg seoTitle "$seo_title" \
  --arg seoDescription "$seo_description" \
  --argjson tags "$tags_json" \
  '{title: $title, excerpt: $excerpt, contentMd: $contentMd, status: $status, tags: $tags, seoTitle: $seoTitle, seoDescription: $seoDescription}')

response=$(curl -sS -X POST "${api_base%/}" \
  -H "Authorization: Bearer ${ARTICLES_TOKEN}" \
  -H 'Content-Type: application/json' \
  --data "$payload")
if ! printf '%s' "$response" | jq -e '.slug and .id' >/dev/null; then
  printf '%s\n' "$response" >&2
  exit 1
fi
printf '%s\n' "$response" | jq '{id, slug, title, status, publishedAt}'
