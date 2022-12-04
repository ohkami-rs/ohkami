#[cfg(any(feature = "postgres", feature = "mysql"))]
#[allow(non_snake_case)]
pub fn useDB<T, F: futures::Future<Output = sqlx::Result<T>>>(
    db_future: F
) -> crate::result::Result<T> {
    async_std::task::block_on(async {
        Ok(db_future.await?)
    })
}