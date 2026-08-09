/**
 * Shared article API client used by every component/route in this package.
 * Reads the PUBLIC_ARTICLES_URL env var that eco resolves for the host
 * composition (falls back to the gateway's /api/articles path).
 */
export const ARTICLES_API = (import.meta.env.PUBLIC_ARTICLES_URL || '/api/articles').replace(/\/$/, '');

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
 * Lightweight markdown-to-HTML renderer used for article bodies. Handles the
 * common subset used by this demo (headings, paragraphs, bold/italic, links,
 * lists, code, blockquotes, images) without pulling in a heavy dependency.
 */
export function renderMarkdown(md = '') {
  const escaped = md
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');

  let html = escaped;

  // Headings
  html = html.replace(/^###### (.*)$/gm, '<h6>$1</h6>');
  html = html.replace(/^##### (.*)$/gm, '<h5>$1</h5>');
  html = html.replace(/^#### (.*)$/gm, '<h4>$1</h4>');
  html = html.replace(/^### (.*)$/gm, '<h3>$1</h3>');
  html = html.replace(/^## (.*)$/gm, '<h2>$1</h2>');
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
