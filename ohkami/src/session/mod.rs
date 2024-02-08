use std::{pin::Pin, sync::Arc};
use crate::__rt__::{AsyncReader, AsyncWriter, Mutex, TcpStream};
use crate::ohkami::router::RadixRouter;
use crate::Request;


pub(crate) struct Session {
    router:     Arc<RadixRouter>,
    connection: TcpStream,
}
impl Session {
    pub(crate) fn new(
        router:     Arc<RadixRouter>,
        connection: TcpStream,
    ) -> Self {
        Self {
            router,
            connection,
        }
    }

    pub(crate) async fn manage(mut self) {
        let connection = &mut self.connection;

        {
            let mut req = Request::init();
            let mut req = unsafe {Pin::new_unchecked(&mut req)};
            req.as_mut().read(connection).await;

            let res = self.router.handle(req.get_mut()).await;
            res.send(connection).await;
        }

        todo!()
    }
}
