use topology::ServerType;

use hyper::service::*;
use crate::{Request, Server};

use tokio::net::TcpListener;

use hyper::service::service_fn;
use hyper::server::conn::http1;
use hyper_util::{TokioIo};


pub struct WebServer{

}

impl  Server for  WebServer{
    type RequestType = ();
    type ResponseType = ();

    fn handle_request(&mut self, request: Self::RequestType) -> Self::ResponseType {
        todo!()
    }

    fn get_sever_type() -> ServerType {
        todo!()
    }

}

impl  WebServer{
    fn new()->Self {
        Self
    }

    pub async fn run(&self, addr: &str) {
        // Bind the address using TcpListener
        let listener = TcpListener::bind(addr).await.expect("Failed to bind address");
        println!("Server running at http://{}/", addr);

        loop {
            // Accept a new connection
            let (stream, _) = listener.accept().await?;

            // Use an adapter to access something implementing `tokio::io` traits as if they implement
            // `hyper::rt` IO traits.
            let io = TokioIo::new(stream);

            // Spawn a new task to handle the connection
            tokio::task::spawn(async move {
                // Finally, we bind the incoming connection to our `hello` service
                if let Err(err) = http1::Builder::new()
                    // `service_fn` converts our function in a `Service`
                    .serve_connection(io, service_fn(WebServer::handle_request))
                    .await
                {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });
        }
    }

}


