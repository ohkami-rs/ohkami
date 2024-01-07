pub trait Repository {
    fn users(&self)    -> impl UsersRepository;
    fn articles(&self) -> impl ArticlesRepository;
    fn comments(&self) -> impl CommentsRepository;
    fn tags(&self)     -> impl TagsRepository;
}


pub trait UsersRepository {
    async fn create(&self);
}

pub struct 


pub trait ArticlesRepository {

}


pub trait CommentsRepository {

}


pub trait TagsRepository {

}
