use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::time::Duration;
use tokio::net::{TcpListener, ToSocketAddrs};

mod router;
mod state;

pub use state::*;

pub async fn run<A: ToSocketAddrs>(
    addr: A,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;

    const CONNECTION_TIMEOUTS: [Duration; 2] = [Duration::from_secs(5), Duration::from_secs(2)];

    loop {
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            let conn = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, service_fn(router::handler));
            tokio::pin!(conn);

            for timeout in CONNECTION_TIMEOUTS {
                tokio::select! {
                    _ = conn.as_mut() => {
                        break;
                    }
                    _ = tokio::time::sleep(timeout) => {
                        conn.as_mut().graceful_shutdown();
                    }
                }
            }
        });
    }
}
