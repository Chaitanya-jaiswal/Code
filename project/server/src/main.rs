use server::web_server::server::WebServer;

#[tokio::main]
async fn main(){
    let ws=  WebServer::new();
    ws.run("http://localhos:9080").await.expect("error starting server");

}