use actix_web::{App, HttpServer};
use log::Level;
use std::env;

mod config;
mod dcds_regist_manage;
mod meta_manage;
mod get_quota_infor;
pub mod response;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    //Initialize the log and set the print level
    simple_logger::init_with_level(Level::Warn).unwrap();

    let path = if args.len() > 1{
        String::from(&args[1])
    }else{
        String::from("127.0.0.1:9001")
    };

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
            .service(get_quota_infor::chang_quota_state)
            .service(get_quota_infor::get_quota_type)
    })
    .bind(path)?
    .run()
    .await
}
