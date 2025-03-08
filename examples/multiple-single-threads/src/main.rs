use ohkami::prelude::*;

fn main() {
    async fn serve(o: Ohkami) -> std::io::Result<()> {
        let socket = tokio::net::TcpSocket::new_v4()?;

        socket.set_reuseport(true)?;
        socket.set_reuseaddr(true)?;
        socket.set_nodelay(true)?;

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

    for _ in 0..(num_cpus::get() - 1/*for main thread*/) {
        std::thread::spawn(|| {
            runtime().block_on(serve(ohkami())).expect("serving error")
        });
    }
    runtime().block_on(serve(ohkami())).expect("serving error")
}

fn ohkami() -> Ohkami {
    Ohkami::new((
        "/".GET(async || {"Hello, world!"}),
    ))
}
