pub struct Path<P: PathType>(P);

pub trait PathType {
    // fn parse
}
// impl PathType for {integer} {}
// impl PathType for String {}
// impl PathType for ({int}, String)
// impl PathType for (String, {int})
// impl PathType for ({int}, {int})
// impl PathType for (String, String)
