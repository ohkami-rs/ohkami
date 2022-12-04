use futures::Future;
use crate::prelude::Result;


#[allow(non_snake_case)]
pub fn useDB<T, F: Future<Output = sqlx::Result<T>>>(
    db_future: F
) -> Result<T> {
    async_std::task::block_on(async {
        Ok(db_future.await?)
    })
}