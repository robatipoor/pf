use std::fs::File;
use std::sync::Arc;

use axum::Router;
use futures_util::pin_mut;
use hyper::body::Incoming;
use hyper::Request;
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::net::TcpListener;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tower_service::Service;

// Async function to serve incoming connections over TLS
pub async fn serve(tcp_listener: TcpListener, router: Router, config: ServerConfig) {
  let tls_acceptor = TlsAcceptor::from(Arc::new(config));
  pin_mut!(tcp_listener);
  // Continuously accept and handle incoming connections
  loop {
    // Wait for a new TCP connection
    let (tcp_stream, addr) = match tcp_listener.accept().await {
      Ok(s) => s,
      Err(err) => {
        tracing::error!("Error during accept TCP connection, Error: {err}");
        continue;
      }
    };

    let tower_service = router.clone();
    let tls_acceptor = tls_acceptor.clone();
    tokio::spawn(async move {
      // Handle TLS handshake
      let tls_stream = match tls_acceptor.accept(tcp_stream).await {
        Ok(s) => s,
        Err(err) => {
          tracing::error!("Error during TLS handshake connection from: {addr}, Error: {err}");
          return;
        }
      };

      // Hyper has its own `AsyncRead` and `AsyncWrite` traits and doesn't use tokio.
      // `TokioIo` converts between them.
      let stream = TokioIo::new(tls_stream);

      // Hyper also has its own `Service` trait and doesn't use tower. We can use
      // `hyper::service::service_fn` to create a hyper `Service` that calls our app through
      // `tower::Service::call`.
      let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
        // We have to clone `tower_service` because hyper's `Service` uses `&self` whereas
        // tower's `Service` requires `&mut self`.
        // We don't need to call `poll_ready` since `Router` is always ready.
        tower_service.clone().call(request)
      });

      if let Err(err) = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
        .serve_connection_with_upgrades(stream, hyper_service)
        .await
      {
        tracing::warn!("Failed serving connection from {addr}, Error: {err}");
      }
    });
  }
}

// Function to create a Rustls ServerConfig from key and cert files
pub fn rustls_server_config(
  key: impl AsRef<std::path::Path>,
  cert: impl AsRef<std::path::Path>,
) -> anyhow::Result<ServerConfig> {
  // Open and read key and cert files
  let mut key_reader = std::io::BufReader::new(File::open(key)?);
  let mut cert_reader = std::io::BufReader::new(File::open(cert)?);

  // Extract private key and certificates from files
  let key = rustls_pemfile::private_key(&mut key_reader)?
    .ok_or_else(|| anyhow::anyhow!("Key is invalid"))?;

  let certs = rustls_pemfile::certs(&mut cert_reader)
    .into_iter()
    .collect::<std::io::Result<Vec<_>>>()?;

  // Configure ServerConfig with extracted key and certificates
  let mut config = ServerConfig::builder()
    .with_no_client_auth()
    .with_single_cert(certs, key)?;

  config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

  Ok(config)
}
