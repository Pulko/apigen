use actix_web::{get, post, delete, web, Responder};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    
    pub id: i32,
    
    pub name: String,
    
}

#[get("/user")]
pub async fn all() -> impl Responder {
    web::Json(vec![
        User {
            
            id: Default::default(),
            
            name: Default::default(),
            
        }
    ])
}

#[get("/user/{id}")]
pub async fn get(id: web::Path<i32>) -> impl Responder {
    format!("Return User with id {}", id)
}

#[post("/user")]
pub async fn create(item: web::Json<User>) -> impl Responder {
    format!("Create new User: {:?}", item)
}

#[delete("/user/{id}")]
pub async fn delete(id: web::Path<i32>) -> impl Responder {
    format!("Delete User with id {}", id)
}
