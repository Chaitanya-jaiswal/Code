use topology::ServerType;

use hyper::service::*;
use crate::{DroneSend, Request, Response, Server};

use tokio::net::TcpListener;

use hyper::service::service_fn;
use hyper::server::conn::http1;
use hyper_util::rt::tokio::TokioIo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebServerRequest {
    ServerType,
    FileList,
    File { file_id: u64, media_ids: Vec<u64> },
    Media { media_id: u64 },
}


impl DroneSend for WebServerRequest {}
impl Request for WebServerRequest {}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebServerResponse {
    ServerType(ServerType),
    FileList { list_length: usize, file_ids: Vec<u64> },
    File { file_size: usize, file: Vec<u8> },
    Media { media_size: usize, media: Vec<u8> },
    ErrorNoFiles,
    ErrorFileNotFound,
    ErrorNoMedia,
    ErrorMediaNotFound,
}

impl DroneSend for WebServerResponse {}
impl Response for WebServerResponse {}

pub struct WebServer{

}

impl  Server for  WebServer{
    type RequestType = WebServerRequest;
    type ResponseType = WebServerResponse;

    fn handle_request(&mut self, request: Self::RequestType) -> Self::ResponseType {

        // create routes for
        ///
        /// C -> S : server_type?
        // S -> C : server_type!(type)
        // C -> S : files_list?
        // S -> C : files_list!(list_length, list_of_file_ids)
        // S -> C : error_no_files!
        // C -> S : file?(file_id, list_length, list_of_media_ids)
        // S -> C : file!(file_size, file)
        // S -> C : error_file_not_found!
        // [Faulty] The communication protocol specifications 13
        // C -> S : media?(media_id)
        // S -> C : media!(media_size, media)
        // S -> C : error_no_media!
        // S -> C : error_media_not_found!
        ///
        todo!()
    }

    fn get_sever_type() -> ServerType {
        todo!()
    }

}

impl  WebServer{
    pub fn new() ->Self {
        Self{}
    }

    pub async fn run(&self, addr: &str)->Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
            // may be switch to crossbeam  here
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


