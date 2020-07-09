use crate::response::ResponseBody;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use log::{info, warn};
use serde::{Deserialize, Serialize};
//数据库相关
use deadpool_postgres::Pool;
use tokio_postgres::row::Row;

//修改货币发行状态
#[derive(Deserialize, Debug)]
pub struct QuotaApproveRequest {
    aid: String,
    extra: serde_json::Value,
    state: String,
}

#[post("/api/quota/approve")]
pub async fn chang_quota_state(
    data: web::Data<Pool>,
    qstr: web::Json<QuotaApproveRequest>,
    req_head: HttpRequest,
) -> impl Responder {
    //获取请求头中的uuid
    let http_head = req_head.headers();
    let head_value = http_head.get("X-CLOUD-USER_ID").unwrap();
    let head_str = head_value.to_str().unwrap();
    //连接数据库句柄
    let conn = data.get().await.unwrap();
    //state值判断
    if (qstr.state == "approve") || (qstr.state == "registe") || (qstr.state == "deined") {
        info!("input state :{:?}", qstr.state);
    } else {
        warn!(
            "request state error:{:?} ,please input the correct state",
            qstr.state
        );
        return HttpResponse::Ok().json(ResponseBody::<()>::new_json_parse_error());
    }
    //修改数据表的值得状态
    match conn
        .query(
            "UPDATE quota_admin SET extra = $1,state = $2,update_time = now() where aid = $3 and cloud_user_id = $4",
            &[&qstr.extra, &qstr.state, &qstr.aid, &head_str],
        )
        .await
    {
        Ok(row) => {
            info!("update success!{:?}", row);
            row
        }
        Err(error) => {
            warn!("chang_quota_state update failde!!{:?}", error);
            return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(Some(
                error.to_string(),
            )));
        }
    };
    HttpResponse::Ok().json(ResponseBody::<()>::new_success(None))
}

//获取额度控制表中不同状态额度值
#[derive(Deserialize, Debug)]
pub struct StatesRequest {
    aid: String,
}

#[derive(Serialize)]
pub struct StatesRespones {
    tatol: i64,   //总额度（额度管理表中状态为delivery的value和）
    issued: i64,  //已发行额度（额度发行表中info信息字段和）
    waitly: i64,  //待发放额度tatol-issued
    recycle: i64, //已回收额度 withdraw
}

#[get("/api/quota/statistics")]
pub async fn get_quota_type(
    data: web::Data<Pool>,
    req: web::Query<StatesRequest>,
    req_head: HttpRequest,
) -> impl Responder {
    //获取请求头中的uuid
    let http_head = req_head.headers();
    let head_value = http_head.get("X-CLOUD-USER_ID").unwrap();
    let head_str = head_value.to_str().unwrap();
    //连接数据库句柄
    let conn = data.get().await.unwrap();
    let mut tatol: i64 = 0; //总额度（额度管理表中状态为delivery的value和）
    let mut issued: i64 = 0; //已发行额度（额度发行表中info信息字段和）
    let mut _waitly: i64 = 0; //待发放额度tatol-issued
    let mut recycle: i64 = 0; //已回收额度 withdraw
    let mut _select_tatol: Vec<Row> = Vec::new();
    let mut _select_issued: Vec<Row> = Vec::new();
    let mut _select_recycle: Vec<Row> = Vec::new();
    let type_delive = String::from("delivery");
    let type_with = String::from("withdraw");
    println!("req: {:?}", req);
    if req.aid.len() > 5 {
        //总额度查询
        _select_tatol = match conn
            .query(
                "SELECT value from quota_admin where aid = $1 AND type = $2 AND cloud_user_id = $3",
                &[&req.aid, &type_delive, &head_str],
            )
            .await
        {
            Ok(row) => {
                info!("electe success: {:?}", row);
                row
            }
            Err(error) => {
                warn!("select failed :{:?}!!", error);
                return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(
                    Some(error.to_string()),
                ));
            }
        };
        if _select_tatol.is_empty() {
            warn!("1.SELECT check get_quota_type failed,please check get_quota_type value");
            return HttpResponse::Ok().json(ResponseBody::<()>::database_build_error());
        }
        //已发额度查询
        _select_issued = match conn
            .query(
                "SELECT (issue_info->'t_obj'->'issue_info') from quota_delivery where aid = $1 and cloud_user_id = $2",
                &[&req.aid, &head_str],
            )
            .await
        {
            Ok(row) => {
                info!("electe success: {:?}", row);
                row
            }
            Err(error) => {
                warn!("select failed :{:?}!!", error);
                return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(
                    Some(error.to_string()),
                ));
            }
        };
        if _select_issued.is_empty() {
            warn!("2.SELECT check get_quota_type failed,please check get_quota_type value");
            return HttpResponse::Ok().json(ResponseBody::<()>::database_build_error());
        }
        //回收额度
        _select_recycle = match conn
            .query(
                "SELECT value from quota_admin where aid = $1 AND type = $2 AND cloud_user_id = $3",
                &[&req.aid, &type_with, &head_str],
            )
            .await
        {
            Ok(row) => {
                info!("electe success: {:?}", row);
                row
            }
            Err(error) => {
                warn!("3.select failed :{:?}!!", error);
                return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(
                    Some(error.to_string()),
                ));
            }
        };
        if _select_recycle.is_empty() {
            warn!("3.SELECT check get_quota_type The amount recovered is empty");
            //  return HttpResponse::Ok().json(ResponseBody::<()>::database_build_error());
        }
    } else {
        //总额度查询
        _select_tatol = match conn
            .query(
                "SELECT value from quota_admin where type = $1 AND cloud_user_id = $2",
                &[&type_delive, &head_str],
            )
            .await
        {
            Ok(row) => {
                info!("electe success: {:?}", row);
                row
            }
            Err(error) => {
                warn!("4.select failed :{:?}!!", error);
                return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(
                    Some(error.to_string()),
                ));
            }
        };
        if _select_tatol.is_empty() {
            warn!("4.SELECT check get_quota_type failed,please check get_quota_type value");
            return HttpResponse::Ok().json(ResponseBody::<()>::database_build_error());
        }
        //已发额度查询
        _select_issued = match conn
            .query(
                "SELECT (issue_info->'t_obj'->'issue_info') from quota_delivery where $1 !='' AND cloud_user_id = $2",
                &[&"aid", &head_str],
            )
            .await
        {
            Ok(row) => {
                info!("electe success: {:?}", row);
                row
            }
            Err(error) => {
                warn!("5.select failed :{:?}!!", error);
                return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(
                    Some(error.to_string()),
                ));
            }
        };
        if _select_issued.is_empty() {
            warn!("5.SELECT check get_quota_type failed,please check get_quota_type value");
            return HttpResponse::Ok().json(ResponseBody::<()>::database_build_error());
        }
        //回收额度
        _select_recycle = match conn
            .query(
                "SELECT value from quota_admin where type = $1 AND cloud_user_id = $2",
                &[&type_with, &head_str],
            )
            .await
        {
            Ok(row) => {
                info!("electe success: {:?}", row);
                row
            }
            Err(error) => {
                warn!("select failed :{:?}!!", error);
                return HttpResponse::Ok().json(ResponseBody::<String>::database_runing_error(
                    Some(error.to_string()),
                ));
            }
        };
        if _select_recycle.is_empty() {
            warn!("6.SELECT check get_quota_type The amount recovered is empty");
            // return HttpResponse::Ok().json(ResponseBody::<()>::database_build_error());
        }
    }
    //获取总额度
    for item in _select_tatol.iter() {
        let value: i64 = item.get(0);
        tatol += value;
    }
    //获取已回收额度
    for item in _select_recycle.iter() {
        let value: i64 = item.get(0);
        recycle += value;
    }
    //获取已发放额度总和
    for item in _select_issued.iter() {
        let issued_vec: Vec<Vec<i64>> = serde_json::from_value(item.get(0)).unwrap();
        for vec_num in issued_vec.iter() {
            let mut number: Vec<i64> = [0; 2].to_vec();
            for (index, quota) in vec_num.iter().enumerate().take(number.len()) {
                number[index] = *quota;
            }
            issued += number[0] * number[1];
        }
    }
    println!(
        "tatol = {}\nissued = {}\nwaitly = {}\nrecycle = {}",
        tatol, issued, _waitly, recycle
    );
    //获取待发放额度
    if tatol >= issued {
        _waitly = tatol - issued;
    } else {
        warn!("The amount issued should not be greater than the total amount!!");
        return HttpResponse::Ok().json(ResponseBody::<()>::new_json_parse_error());
    }
    let all_quota = StatesRespones {
        tatol,
        issued,
        waitly: _waitly,
        recycle,
    };
    HttpResponse::Ok().json(ResponseBody::<StatesRespones>::new_success(Some(all_quota)))
}
