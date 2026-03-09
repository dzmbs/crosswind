pub mod date;
pub mod error;
pub mod fetch;
pub mod model;
pub mod output;
pub mod parse;
pub mod query;

use error::CrosswindError;
use model::SearchResult;
use query::QueryParams;

pub async fn search(
    params: &QueryParams,
    destination: &str,
    timeout_secs: u64,
) -> Result<SearchResult, CrosswindError> {
    let url = query::build_url(params, destination);
    let html = fetch::fetch_html(&url, timeout_secs).await?;
    parse::parse(&html)
}
