/// File in `multipart/form-data`, `multipart/mixed` requests
/// 
/// <br>
/// 
/// *expected_usage.rs*
/// 
/// ---
/// ```ignore
/// #[Payload(Multipart/D)]
/// struct SignUpForm<'req> {
///     #[serde(rename = "user-name")]
///     user_name:  &'req str,
/// 
///     password:   &'req str,
///     
///     #[serde(rename = "user-icon")]
///     user_icon:  Option<File<'req>>,
/// 
///     #[serde(rename = "pet-photos")]
///     pet_photos: Vec<File<'req>>,
/// }
/// 
/// async fn sign_up(
///     form: SignUpForm<'_>,
///     pool: Memory<'_, PgPool>,
/// ) -> Result<OK<String>, APIError> {
///     let SignUpForm {
///         user_name, password, user_icon, pet_photos
///     } = form;
/// 
///     let password = hash(password);
/// 
///     if sqlx::query_scalor!("
///         SELECT EXISTS (SELECT * FROM users
///         WHERE name = $1 AND password = $2)
///     ", user_name, password).fetch_one(*pool).await? {
///         return Err(APIError::UserAlrleadyExists)
///     }
/// 
///     let user_id = sqlx::query!("
///         INSERT INTO users (name, password, icon) VALUES ($1, $2, $3)
///         RETURNING id
///     ", user_name, password, icon).fetch_one(*pool).await?;
///     
///     if pet_photos.len() > 0 {
///         sqlx::query!("
///             INSERT INTO pet_photos (raw) SELECT UNNEST($1::bytea[])
///         ", pet_photos).execute(*pool).await?;
///     }
/// 
///     Ok(OK(isse_jwt_token(user_id)))
/// }
/// ```
/// ---
pub struct File {

}



struct FileInner {

}
