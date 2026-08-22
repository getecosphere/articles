use axum::{extract::{Path, State}, response::Html, routing::get, Router};
use serde::Deserialize;

#[derive(Clone)]
struct AppState { client: reqwest::Client, api: String, site: String }

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Article {
    slug: String,
    title: String,
    excerpt: Option<String>,
    content_md: String,
    tags: Vec<String>,
    seo_title: Option<String>,
    seo_description: Option<String>,
}

fn escape(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn document(title: &str, description: &str, body: String, site: &str, path: &str) -> Html<String> {
    let canonical = format!("{}{}", site.trim_end_matches('/'), path);
    Html(format!(r#"<!doctype html><html lang="id"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{}</title><meta name="description" content="{}"><link rel="canonical" href="{}"><meta property="og:title" content="{}"><meta property="og:description" content="{}"><meta property="og:type" content="article"><style>
*{{box-sizing:border-box}}body{{margin:0;background:#f8f7f2;color:#17241d;font:16px/1.65 Inter,ui-sans-serif,system-ui,-apple-system,sans-serif}}a{{color:inherit}}.shell{{max-width:1100px;margin:auto;padding:28px 24px 72px}}nav{{display:flex;justify-content:space-between;align-items:center;padding:4px 0 56px}}.brand{{font:700 25px Georgia,serif;text-decoration:none}}.brand i{{color:#558563;font-style:normal}}.back{{color:#557361;text-decoration:none;font-weight:650}}header{{max-width:720px;margin-bottom:36px}}h1{{font:700 clamp(36px,6vw,68px)/1.05 Georgia,serif;letter-spacing:-.04em;margin:0 0 18px}}.lead{{font-size:19px;color:#526158;margin:0}}.grid{{display:grid;grid-template-columns:repeat(auto-fit,minmax(250px,1fr));gap:16px}}.card{{background:#fff;border:1px solid #e5e5dd;border-radius:16px;padding:24px;text-decoration:none;transition:transform .2s,box-shadow .2s}}.card:hover{{transform:translateY(-3px);box-shadow:0 14px 30px #14251b12}}.tag{{font-size:12px;color:#63816a;font-weight:750;text-transform:uppercase;letter-spacing:.08em}}.card h2{{font:700 27px/1.12 Georgia,serif;margin:12px 0}}.card p{{color:#637068;margin:0}}.story{{max-width:760px;margin:auto;background:#fff;border:1px solid #e5e5dd;border-radius:18px;padding:clamp(26px,6vw,66px)}}.story h1{{font-size:clamp(34px,5vw,60px)}}.story h2{{font:700 28px/1.2 Georgia,serif;margin-top:38px}}.story p{{font-size:18px;color:#38473d}}.story ul{{padding-left:24px}}.empty{{padding:32px;background:#fff;border:1px solid #e5e5dd;border-radius:16px;color:#617066}}@media(max-width:600px){{nav{{padding-bottom:38px}}.shell{{padding:22px 18px 54px}}}}
</style></head><body><main class="shell">{}</main></body></html>"#, escape(title), escape(description), escape(&canonical), escape(title), escape(description), body))
}

async fn list(State(state): State<AppState>) -> Html<String> {
    let response = state.client.get(&state.api).send().await;
    let articles = match response { Ok(r) if r.status().is_success() => r.json::<Vec<Article>>().await.unwrap_or_default(), _ => Vec::new() };
    let cards = if articles.is_empty() {
        "<p class=\"empty\">Belum ada artikel yang diterbitkan.</p>".to_string()
    } else { articles.iter().map(|a| {
        let tag = a.tags.first().map(String::as_str).unwrap_or("self development");
        format!("<a class=\"card\" href=\"/articles/{}\"><span class=\"tag\">{}</span><h2>{}</h2><p>{}</p></a>", escape(&a.slug), escape(tag), escape(&a.title), escape(a.excerpt.as_deref().unwrap_or("Baca dan praktikkan pelan-pelan.")))
    }).collect::<Vec<_>>().join("") };
    let body = format!("<nav><a class=\"brand\" href=\"/\">Leadme<i>.</i></a><a class=\"back\" href=\"/dashboard\">Dashboard</a></nav><header><h1>Ruang untuk bertumbuh dengan sadar.</h1><p class=\"lead\">Bacaan tentang Seven Habits dan praktik kecil untuk hidup yang lebih selaras.</p></header><section class=\"grid\">{cards}</section>");
    document("Artikel · Leadme", "Bacaan self development dan Seven Habits untuk kehidupan yang lebih terarah.", body, &state.site, "/articles")
}

fn markdown(markdown: &str) -> String {
    let mut output = String::new();
    let mut in_list = false;
    for raw in markdown.lines() {
        let line = escape(raw.trim());
        if line.is_empty() { if in_list { output.push_str("</ul>"); in_list = false; } continue; }
        if let Some(text) = line.strip_prefix("# ") { output.push_str(&format!("<h1>{text}</h1>")); }
        else if let Some(text) = line.strip_prefix("## ") { output.push_str(&format!("<h2>{text}</h2>")); }
        else if let Some(text) = line.strip_prefix("- ") { if !in_list { output.push_str("<ul>"); in_list = true; } output.push_str(&format!("<li>{text}</li>")); }
        else { output.push_str(&format!("<p>{}</p>", line.replace("**", ""))); }
    }
    if in_list { output.push_str("</ul>"); }
    output
}

async fn detail(Path(slug): Path<String>, State(state): State<AppState>) -> Html<String> {
    let url = format!("{}/slug/{}", state.api, slug);
    let article = match state.client.get(url).send().await {
        Ok(response) if response.status().is_success() => response.json::<Article>().await.ok(),
        _ => None,
    };
    match article {
        Some(article) => {
            let description = article.seo_description.clone().or(article.excerpt.clone()).unwrap_or_else(|| "Baca artikel Leadme.".into());
            let title = article.seo_title.clone().unwrap_or_else(|| format!("{} · Leadme", article.title));
            let body = format!("<nav><a class=\"brand\" href=\"/\">Leadme<i>.</i></a><a class=\"back\" href=\"/articles\">← Semua artikel</a></nav><article class=\"story\">{}</article>", markdown(&article.content_md));
            document(&title, &description, body, &state.site, &format!("/articles/{}", article.slug))
        }
        None => document("Artikel tidak ditemukan · Leadme", "Artikel ini tidak tersedia.", "<nav><a class=\"brand\" href=\"/\">Leadme<i>.</i></a><a class=\"back\" href=\"/articles\">← Semua artikel</a></nav><p class=\"empty\">Artikel ini belum tersedia atau sudah tidak dipublikasikan.</p>".into(), &state.site, "/articles"),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().json().with_env_filter("info").init();
    let raw_api = std::env::var("ARTICLES_API_URL").unwrap_or_else(|_| "http://127.0.0.1:8080/api".into()).trim_end_matches('/').to_string();
    let api = if raw_api.ends_with("/articles") { raw_api } else { format!("{raw_api}/articles") };
    let state = AppState { client: reqwest::Client::new(), api, site: std::env::var("PUBLIC_SITE_URL").unwrap_or_else(|_| "https://leadme.getecosphere.app".into()) };
    let app = Router::new().route("/articles", get(list)).route("/articles/", get(list)).route("/articles/:slug", get(detail)).with_state(state);
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.expect("bind articles-ui");
    tracing::info!(service = "articles-ui", port = %port, "starting");
    axum::serve(listener, app).await.expect("serve articles-ui");
}
