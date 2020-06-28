use actix_cors::Cors;
use actix_web::{App, HttpServer};
use clap::ArgMatches;
use log::Level;
use std::env;

mod config;
mod config_command;
mod dcds_regist_manage;
mod get_quota_infor;
mod meta_manage;
mod get_quota_infor;
pub mod response;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    //Initialize the log and set the print level
    simple_logger::init_with_level(Level::Warn).unwrap();

    let mut _path: String = String::new();
    let matches: ArgMatches = config_command::get_command();
    if let Some(d) = matches.value_of("cms") {
        _path = d.to_string();
    } else {
        _path = String::from("127.0.0.1:9001");
    }

    HttpServer::new(|| {
        App::new()
            .wrap(Cors::new().supports_credentials().finish())
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
    .bind(_path)?
    .run()
    .await
}
