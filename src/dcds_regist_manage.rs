use crate::config::ConfigPath;
use crate::response::ResponseBody;
use asymmetric_crypto::hasher::{sha3::Sha3, sm3::Sm3};
use actix_web::{post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use rand::thread_rng;
use chrono::prelude::*;

#[derive(Deserialize, Debug)]
pub struct DcdsRegistRequest{
    cert: String,
    extra: serde_json::Value,
}

#[derive(Serialize, Debug)]
pub struct DcdsRegistResponse{
    cert: String,
    aid: String,
}

#[post("/api/dcds")]
pub async fn dcds_reg_manage(data: web::Data<Pool>,
    qstr: web::Json<DcdsRegistRequest>
) -> impl Responder {
    //数据库连接请求句柄获取
    let conn = data.get().await.unwrap();

    //用于二次sm3的时间戳
    let local_time = Local::now();
    //用于生成sm3的随机值
    let mut rng = thread_rng();
    //use Sm3算法实现hash转换
    let mut uid_hasher = Sm3::default();
    uid_hasher.update(&qstr.cert);
    uid_hasher.update(&local_time);
    uid_hasher.update(&rng);
    let uid_str = uid_hasher.finalize().encode_hex(); 

    //数据库存储操作
    //状态值
    let state = String::from("begin");
    let inert_statement = conn
        .prepare(
            "INSERT INTO agents (id, cert, extra, state, create_time, update_time) VALUES ($1,
                $2, $3, $4, now(), now())",
        ).await.unwrap();
    conn.execute(&inert_statement, &[&uid_str, &qstr.cert, &qstr.extra, &state])
        .await.unwrap();    

    //返回响应字段
    HttpResponse::Ok().json(ResponseBody::new_success(Some(DcdsRegistResponse {
        cert: qstr.cert.clone(),
        aid: uid_str,
    })))
}