use trellara_pg_capture::source_safety_read_only_queries;

use crate::html_escape::push_html_escaped;

pub(crate) fn push_query_appendix(html: &mut String) {
    html.push_str("<section class=\"card\"><h2>Exact Read-Only Queries</h2>");
    html.push_str("<p>These are the query templates embedded in trellara-check. Parameterized queries bind the values listed below.</p>");
    for query in source_safety_read_only_queries() {
        html.push_str("<details open><summary>");
        push_html_escaped(html, query.name);
        html.push_str("<small>");
        push_html_escaped(html, query.when);
        html.push_str("</small></summary><p><strong>Parameters:</strong> ");
        push_html_escaped(html, query.parameters);
        html.push_str("</p><pre><code>");
        push_html_escaped(html, query.sql.trim());
        html.push_str("</code></pre></details>");
    }
    html.push_str("</section>");
}
