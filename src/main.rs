use actix_web::{App, HttpServer};
use log::Level;

mod config;
mod dcds_regist_manage;
mod meta_manage;
pub mod response;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    //Initialize the log and set the print level
    simple_logger::init_with_level(Level::Warn).unwrap();

    HttpServer::new(|| {
        App::new()
            .data(config::get_db())
            .data(config::ConfigPath::default())
            .service(meta_manage::new_cert)
            .service(meta_manage::update_cert)
            .service(meta_manage::get_cert)
            .service(dcds_regist_manage::dcds_reg_manage)
            .service(dcds_regist_manage::new_quota_manage)
            .service(dcds_regist_manage::get_dcds_allquota)
    })
    .bind("127.0.0.1:8077")?
    .run()
    .await
}
