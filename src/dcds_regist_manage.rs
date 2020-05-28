use crate::config::ConfigPath;
use crate::response::ResponseBody;
use actix_web::{post, web, HttpResponse, Responder};
use asymmetric_crypto::hasher::sha3::Sha3;
use asymmetric_crypto::hasher::sm3::Sm3;
use asymmetric_crypto::keypair;
use asymmetric_crypto::prelude::Keypair;
use chrono::prelude::*;
use common_structure::issue_quota_request::IssueQuotaRequestWrapper;
use dislog_hal::{Bytes, Hasher};
use hex::{FromHex, ToHex};
use kv_object::prelude::KValueObject;
use kv_object::sm2::KeyPairSm2;
use log::{info, warn};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::prelude::*;
//数据库相关
use deadpool_postgres::Pool;

#[derive(Deserialize, Debug)]
pub struct DcdsRegistRequest {
    cert: String,
    extra: serde_json::Value,
}

#[derive(Serialize, Debug)]
pub struct DcdsRegistResponse {
    cert: String,
    aid: String,
}

#[post("/api/dcds")]
pub async fn dcds_reg_manage(
    data: web::Data<Pool>,
    qstr: web::Json<DcdsRegistRequest>,
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
    uid_hasher.update(&local_time.to_string());
    uid_hasher.update(&(rng.gen::<[u8; 32]>()));
    let uid_str = uid_hasher.finalize().encode_hex::<String>();

    //数据库存储操作
    //状态值
    let state = String::from("begin");
    let insert_statement = conn
        .prepare(
            "INSERT INTO agents (id, cert, extra, state, create_time, update_time) VALUES ($1,
                $2, $3, $4, now(), now())",
        )
        .await
        .unwrap();
    conn.execute(
        &insert_statement,
        &[&uid_str, &qstr.cert, &qstr.extra, &state],
    )
    .await
    .unwrap();

    //返回响应字段
    HttpResponse::Ok().json(ResponseBody::new_success(Some(DcdsRegistResponse {
        cert: qstr.cert.clone(),
        aid: uid_str,
    })))
}

#[derive(Deserialize, Debug)]
pub struct QuotaManageRequest {
    aid: String,
    value: i64,
    #[serde(rename = "type")]
    ttype: String,
    extra: serde_json::Value,
}

#[post("/api/dcds/{id}/quota/")]
pub async fn new_quota_manage(
    data: web::Data<Pool>,
    qstr: web::Json<QuotaManageRequest>,
) -> impl Responder {
    //数据库连接请求句柄获取
    let conn = data.get().await.unwrap();

    //用于二次sm3的时间戳
    let local_time = Local::now();
    //用于生成sm3的随机值
    let mut rng = thread_rng();
    //use Sm3算法实现hash转换
    let mut uid_hasher = Sm3::default();
    uid_hasher.update(&qstr.aid);
    uid_hasher.update(&qstr.value.to_string());
    uid_hasher.update(&qstr.ttype);
    uid_hasher.update(&local_time.to_string());
    uid_hasher.update(&(rng.gen::<[u8; 32]>()));
    let uid_str = uid_hasher.finalize().encode_hex::<String>();

    //数据库插入数据
    //状态值
    let state = String::from("registe");
    if (qstr.ttype == "withdraw") || (qstr.ttype == "delivery") {
        info!("input type :{:?}", qstr.ttype);
    } else {
        warn!(
            "request type error:{:?} ,please input withdraw or delivery",
            qstr.ttype
        );
        return HttpResponse::Ok().json(ResponseBody::<()>::new_json_parse_error());
    }
    let insert_statement = conn
        .prepare(
            "INSERT INTO quota_admin (id, aid, extra, value, type, state, create_time,
                update_time) VALUES ($1, $2, $3, $4, $5, $6, now(), now())",
        )
        .await
        .unwrap();
    conn.execute(
        &insert_statement,
        &[
            &uid_str,
            &qstr.aid,
            &qstr.extra,
            &qstr.value,
            &qstr.ttype,
            &state,
        ],
    )
    .await
    .unwrap();

    //返回响应
    HttpResponse::Ok().json(ResponseBody::<()>::new_success(None))
}

///额度请求结构体
#[derive(Deserialize, Debug)]
pub struct DcdsQuotaRequest {
    aid: String,
    issue: String,
}

#[post("/api/dcds/qouta_issue/")]
pub async fn get_dcds_allquota(
    data: web::Data<Pool>,
    config: web::Data<ConfigPath>,
    qstr: web::Json<DcdsQuotaRequest>,
) -> impl Responder {
    //连接到数据库获取连接句柄
    let conn = data.get().await.unwrap();
    let mut rng = thread_rng();
    //read file for get seed
    let mut file = match File::open(&config.meta_path).await {
        Ok(f) => {
            info!("{:?}", f);
            f
        }
        Err(e) => {
            warn!("file open failed: {:?}", e);
            return HttpResponse::Ok().json(ResponseBody::<()>::new_file_error());
        }
    };
    //read json file to string
    let mut contents = String::new();
    match file.read_to_string(&mut contents).await {
        Ok(s) => {
            info!("{:?}", s);
            s
        }
        Err(e) => {
            warn!("read file to string failed:{:?}", e);
            return HttpResponse::Ok().json(ResponseBody::<()>::new_str_conver_error());
        }
    };
    //deserialize to the specified data format
    let keypair_value: keypair::Keypair<
        [u8; 32],
        Sha3,
        dislog_hal_sm2::PointInner,
        dislog_hal_sm2::ScalarInner,
    > = match serde_json::from_str(&contents) {
        Ok(de) => {
            info!("{:?}", de);
            de
        }
        Err(e) => {
            warn!("Keypair generate failed:{:?}", e);
            return HttpResponse::Ok().json(ResponseBody::<()>::new_str_conver_error());
        }
    };
    //pass encode hex conversion get seed
    let seed: [u8; 32] = keypair_value.get_seed();
    //get  digital signature
    let keypair_sm2: KeyPairSm2 = KeyPairSm2::generate_from_seed(seed).unwrap();

    //入参额度请求反序列化得到指定格式的值
    let deser_vec = Vec::<u8>::from_hex(&qstr.issue).unwrap();
    let mut issue_quota = IssueQuotaRequestWrapper::from_bytes(&deser_vec).unwrap();

    //验证签名
    if issue_quota.verfiy_kvhead().is_ok() {
        info!("true");
    } else {
        warn!("quota issue request verfiy check failed");
        return HttpResponse::Ok().json(ResponseBody::<()>::new_json_parse_error());
    }
    //重新签名
    issue_quota.fill_kvhead(&keypair_sm2, &mut rng).unwrap();
    let jsonb_issue = serde_json::to_value(&issue_quota).unwrap();
    let response_data = issue_quota.to_bytes().encode_hex::<String>();

    //用于二次sm3的时间戳
    let local_time = Local::now();
    //use Sm3算法实现hash转换
    let mut uid_hasher = Sm3::default();
    uid_hasher.update(&qstr.aid);
    uid_hasher.update(&qstr.issue);
    uid_hasher.update(&local_time.to_string());
    uid_hasher.update(&(rng.gen::<[u8; 32]>()));
    let uid_str = uid_hasher.finalize().encode_hex::<String>();
    //将数据插入数据库
    let insert_statement = conn
        .prepare(
            "INSERT INTO quota_delivery (id, aid, issue, issue_info, create_time, update_time) VALUES 
            ($1, $2, $3, $4, now(), now())",
        ).await.unwrap();
    conn.execute(
        &insert_statement,
        &[&uid_str, &qstr.aid, &qstr.issue, &jsonb_issue],
    )
    .await
    .unwrap();

    HttpResponse::Ok().json(ResponseBody::new_success(Some(response_data)))
}
