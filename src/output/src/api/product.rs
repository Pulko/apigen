use actix_web::{get, post, delete, web, Responder};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    
    pub id: i32,
    
    pub name: String,
    
    pub price: f64,
    
}

#[get("/product")]
pub async fn all() -> impl Responder {
    web::Json(vec![
        Product {
            
            id: Default::default(),
            
            name: Default::default(),
            
            price: Default::default(),
            
        }
    ])
}

#[get("/product/{id}")]
pub async fn get(id: web::Path<i32>) -> impl Responder {
    format!("Return Product with id {}", id)
}

#[post("/product")]
pub async fn create(item: web::Json<Product>) -> impl Responder {
    format!("Create new Product: {:?}", item)
}

#[delete("/product/{id}")]
pub async fn delete(id: web::Path<i32>) -> impl Responder {
    format!("Delete Product with id {}", id)
}
