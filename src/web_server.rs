use axum::{http::Response, response::IntoResponse};
use tokio::task::JoinHandle;
use axum::{routing::get, Router, extract::Path,http::StatusCode};
pub use base64::{engine::general_purpose::URL_SAFE,Engine as _};
use axum::body::Body;
use tokio_util::io::ReaderStream;
pub type Error = Box<dyn std::error::Error + Send + Sync>;

// use 
pub async fn web_server()->JoinHandle<()>{
    let app=axum::Router::new()
        .route("/{id}",axum::routing::get(get_img));



    let listener=tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();


   tokio::spawn(
        async move{
            axum::serve(listener, app).await.unwrap();
        }
    )

}


async fn get_img(Path(id): Path<String>)->impl IntoResponse {

    // base64::decode(id).unwrap();

    // URL_SAFE.decode(id)?;

    match URL_SAFE.decode(id) {
        Ok(path_vec)=>{
            match String::from_utf8(path_vec) {
                Ok(path)=>{
                    match tokio::fs::File::open(path).await {
                        Ok(file)=>{
                            let stream = ReaderStream::new(file); // 将 File 转为流
                            let body = Body::from_stream(stream);
                            Response::builder().status(StatusCode::OK).body(body).unwrap()
                        },
                        Err(_)=>Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("NOT FOUND")).unwrap()
                    }

                    
                }
                Err(_)=>{
                    Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("NOT FOUND")).unwrap()
                }
            }
        }
        Err(_)=>{
            Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("NOT FOUND")).unwrap()
        }
    }
    // let path=String::from_utf8(path_vec).unwrap();




   
   

}