use actix_web::{App, HttpServer};

mod api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
        
            .service(api::user::all)
            .service(api::user::get)
            .service(api::user::create)
            .service(api::user::delete)
        
            .service(api::product::all)
            .service(api::product::get)
            .service(api::product::create)
            .service(api::product::delete)
        
            .service(api::order::all)
            .service(api::order::get)
            .service(api::order::create)
            .service(api::order::delete)
        
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
