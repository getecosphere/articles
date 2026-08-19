/**
 * Shared article API client used by every component/route in this package.
 * Reads the ARTICLES_API_URL env var that eco fills for the estate (falls back
 * to the gateway's /api/articles path). In dev, PUBLIC_ARTICLES_URL still wins
 * for composition builds.
 */
export const ARTICLES_API = (process.env.ARTICLES_API_URL || import.meta.env.PUBLIC_ARTICLES_URL || '/api/articles').replace(/\/$/, '');

export function articleUrl(slug) {
  return `/articles/${slug}/`;
}

/** List published articles (public endpoint). */
export async function fetchArticles() {
  const res = await fetch(`${ARTICLES_API}`);
  if (!res.ok) throw new Error('Unable to load articles');
  return res.json();
}

/** Fetch one published article by slug (public endpoint). */
export async function fetchArticleBySlug(slug) {
  const res = await fetch(`${ARTICLES_API}/slug/${encodeURIComponent(slug)}`);
  if (!res.ok) return null;
  return res.json();
}

/** Absolute URL for a path, built from the host's public site URL. */
export function absolute(siteUrl, path) {
  if (!siteUrl) return path;
  return `${siteUrl.replace(/\/$/, '')}${path.startsWith('/') ? path : `/${path}`}`;
}

/**
 * Convert a heading's plain text into a stable URL anchor id. The same slug
 * is used both when rendering the article body (the `id` attribute on each
 * heading) and when building the on-page table of contents, so TOC links
 * always land exactly on the matching heading.
 */
export function headingSlug(text = '') {
  return text
    .toLowerCase()
    .replace(/&amp;/g, 'and')
    .replace(/[*_`]/g, '')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

/**
 * Extract the document structure (h2–h4 headings) from markdown content, in
 * document order, with the same anchor ids `renderMarkdown` assigns. Used by
 * the on-page left navigation so readers can jump between sections.
 */
export function extractHeadings(md = '') {
  const headings = [];
  for (const line of md.split('\n')) {
    const match = line.match(/^(#{2,4})\s+(.*)$/);
    if (!match) continue;
    const level = match[1].length;
    const text = match[2].trim();
    if (!text) continue;
    const escaped = text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/[*_`]/g, '');
    headings.push({ id: headingSlug(escaped), text, level });
  }
  return headings;
}

/**
 * Lightweight markdown-to-HTML renderer used for article bodies. Handles the
 * common subset used by this demo (headings, paragraphs, bold/italic, links,
 * lists, code, blockquotes, images) without pulling in a heavy dependency.
 * Headings h2–h4 get an `id` anchor so the on-page TOC can link to them.
 */
export function renderMarkdown(md = '') {
  const escaped = md
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');

  let html = escaped;

  // Headings (with anchors on h2–h4, matching extractHeadings)
  html = html.replace(/^###### (.*)$/gm, '<h6>$1</h6>');
  html = html.replace(/^##### (.*)$/gm, '<h5>$1</h5>');
  html = html.replace(/^#### (.*)$/gm, (_m, t) => `<h4 id="${headingSlug(t)}">${t}</h4>`);
  html = html.replace(/^### (.*)$/gm, (_m, t) => `<h3 id="${headingSlug(t)}">${t}</h3>`);
  html = html.replace(/^## (.*)$/gm, (_m, t) => `<h2 id="${headingSlug(t)}">${t}</h2>`);
  html = html.replace(/^# (.*)$/gm, '<h1>$1</h1>');

  // Code blocks (fenced)
  html = html.replace(/```(\w*)\n([\s\S]*?)```/g, (_m, _lang, code) => `<pre><code>${code.replace(/\n$/, '')}</code></pre>`);

  // Inline code
  html = html.replace(/`([^`]+)`/g, '<code>$1</code>');

  // Bold / italic
  html = html.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
  html = html.replace(/__([^_]+)__/g, '<strong>$1</strong>');
  html = html.replace(/(^|[^*])\*([^*\n]+)\*/g, '$1<em>$2</em>');

  // Images
  html = html.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, '<img src="$2" alt="$1" loading="lazy" />');

  // Links
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>');

  // Blockquotes
  html = html.replace(/^&gt; (.*)$/gm, '<blockquote>$1</blockquote>');

  // Lists
  html = html.replace(/((?:^[-*] .*\n?)+)/gm, (block) => {
    const items = block
      .split('\n')
      .filter((l) => l.trim())
      .map((l) => `<li>${l.replace(/^[-*] /, '')}</li>`)
      .join('');
    return `<ul>${items}</ul>`;
  });

  // Paragraphs (lines not consumed by block elements)
  html = html.replace(/^([^<][^\n]*)$/gm, '<p>$1</p>');

  return html;
}
