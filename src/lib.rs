use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};

pub mod configuration;
pub mod routes;
pub mod startup;
