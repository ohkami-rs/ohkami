use super::*;

impl Ohkami {
    #[cfg(feature = "__rt_native__")]
    pub(crate) async fn howl_core<T>(
        self,
        bind: impl __rt__::IntoTcpListener<T>,
        config: crate::Config,
        #[cfg(feature = "tls")] tls_config: Option<rustls::ServerConfig>,
    ) {
        let (router, _) = self.into_router().finalize();
        #[cfg_attr(
            not(feature = "__rt_threaded__"),
            allow(clippy::arc_with_non_send_sync)
        )]
        let router = Arc::new(router);

        let mut listener = bind.into_tcp_listener().await;
        let (wg, ctrl_c) = (sync::WaitGroup::new(), sync::CtrlC::new());

        #[cfg(feature = "tls")]
        let tls_acceptor = tls_config.map(|it| anysc_rustls::TlsAcceptor::from(Arc::new(it)));

        crate::INFO!("start serving on {}", listener.local_addr().unwrap());
        while let Some(accept) = ctrl_c.until_interrupt(listener.accept()).await {
            let Ok(conn) = accept else {
                continue;
            };
            // Manually impl Send + Sync, bcs `__rt_threaded__` is disabled
            #[allow(warnings)]
            {
                unsafe impl Send for crate::router::r#final::Router {}
                unsafe impl Sync for crate::router::r#final::Router {}
            }
            #[cfg(feature = "tls")]
            let tls_acceptor = tls_acceptor.clone();
            let router = router.clone();

            let wg = wg.add();
            let accept = || async move {
                let address = conn.peer_addr()?;
                let stream = conn.connect().await?;

                #[cfg(feature = "tls")]
                let stream: session::Connection = match tls_acceptor {
                    None => stream.into(),
                    Some(tls_acceptor) => match tls_acceptor.accept(stream).await {
                        Ok(tls_stream) => tls_stream.into(),
                        Err(e) => {
                            crate::ERROR!("TLS accept error: {e}");
                            return Ok(());
                        }
                    },
                };

                let session = session::Session::new(config, stream, address.ip(), router);
                session.manage().await;
                wg.done();

                std::io::Result::Ok(())
            };
            nio::spawn_pinned(accept);
        }

        crate::INFO!("interrupted, trying graceful shutdown...");
        drop(listener);

        crate::INFO!("waiting {} session(s) to finish...", wg.count());
        wg.await;
    }
}
