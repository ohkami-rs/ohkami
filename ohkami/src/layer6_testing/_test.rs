use crate::Status;
use crate::__rt__;
use crate::prelude::*;

use crate::testing::*;

#[__rt__::test] async fn testing_example() {
    let simple_ohkami = Ohkami::new(());
    assert_eq!(
        simple_ohkami.oneshot(TestRequest::GET("/")
            .json_lit(r#"
                {}
            "#)).await.status,
        Status::NotFound
    );
}
