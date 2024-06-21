use actix_web::{get, post, delete, web, Responder};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    
    pub id: i32,
    
    pub user_id: i32,
    
    pub product_id: i32,
    
    pub quantity: i32,
    
}

#[get("/order")]
pub async fn all() -> impl Responder {
    web::Json(vec![
        Order {
            
            id: Default::default(),
            
            user_id: Default::default(),
            
            product_id: Default::default(),
            
            quantity: Default::default(),
            
        }
    ])
}

#[get("/order/{id}")]
pub async fn get(id: web::Path<i32>) -> impl Responder {
    format!("Return Order with id {}", id)
}

#[post("/order")]
pub async fn create(item: web::Json<Order>) -> impl Responder {
    format!("Create new Order: {:?}", item)
}

#[delete("/order/{id}")]
pub async fn delete(id: web::Path<i32>) -> impl Responder {
    format!("Delete Order with id {}", id)
}
