use crate::api::{ApiOperationQuery, ApiResponse};
use std::future::Future;

pub trait PaginableOperationQuery {
    fn set_page_id(&mut self, page_id: Option<u32>);
}

/// Unfold a paginated API response
///
/// This function accepts a closure named "operation", that accepts a query, and returns a paginated
/// api response. It calls that closure for every api page, aggregates the response content and
/// returns it.
pub async fn unfold_api_list<'a, T, Q, Operation, Fut>(
    client: &'a reqwest::Client,
    query: Q,
    operation: Operation,
) -> Result<Vec<T>, reqwest::Error>
where
    Q: ApiOperationQuery + PaginableOperationQuery + Clone,
    Operation: Fn(&'a reqwest::Client, Q) -> Fut,
    Fut: Future<Output = Result<ApiResponse<Vec<T>>, reqwest::Error>> + Send,
{
    // Declare a vector for holding the results from the calls to the closure
    let mut aggregator = vec![];

    // Declare a variable for holding the next page id
    let mut next_page = Some(1);

    // Call the closure iteratively until the pagination is exhausted
    loop {
        // Update the query pagination to the next page id.
        let mut query = query.clone();
        query.set_page_id(next_page);

        // Call the closure
        let response = operation(client, query).await?;

        // Add the response data to the aggregator
        aggregator.extend(response.data);

        // Update the pagination
        next_page = match response.meta.current_page < response.meta.last_page {
            true => Some(response.meta.current_page + 1),
            false => None,
        };

        // Continue until all pages are exhausted
        if next_page.is_none() {
            break;
        }
    }

    Ok(aggregator)
}
