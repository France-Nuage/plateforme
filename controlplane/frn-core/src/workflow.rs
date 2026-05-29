use std::future::Future;

use sqlx::PgConnection;

pub trait WorkflowScheduler<P>: Clone + Send + Sync {
    fn schedule<'a>(
        &'a self,
        conn: &'a mut PgConnection,
        params: P,
    ) -> impl Future<Output = Result<(), String>> + Send + 'a;
}
