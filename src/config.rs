use deadpool_postgres::{Manager, Pool};
use tokio_postgres::{ Config, NoTls};

#[derive(Clone)]
pub struct ConfigPath {
    pub meta_path: String,
}

impl Default for ConfigPath {
    fn default() -> Self{
        Self {
            meta_path: String::from("./meta_manage.json"),
        }
    }
}

///配置数据库连接信息
pub fn get_db() -> Pool {
    //配置数据库
    let mut cfg = Config::new();
    cfg.host("localhost");//数据库地址
    cfg.user("postgres");//数据库用户名称
    cfg.password("postgres");//数据库密码
    cfg.dbname("centralmanagesystem");//数据库名称
    let mgr = Manager::new(cfg, NoTls); //生成一个数据库管理池
    Pool::new(mgr, 8)//设置最大连接池
}