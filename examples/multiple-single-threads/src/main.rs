use ohkami::prelude::*;

fn main() {
    async fn serve(o: Ohkami) -> std::io::Result<()> {
        let socket = tokio::net::TcpSocket::new_v4()?;

        socket.bind("0.0.0.0:8000".parse().unwrap())?;

        let listener = socket.listen(1024)?;

        o.howl(listener).await;

        Ok(())
    }

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    runtime().block_on(serve(ohkami())).expect("serving error")
}

fn ohkami() -> Ohkami {
    Ohkami::new((
        "/".GET(async || {"Hello, world!"}),
    ))
}
