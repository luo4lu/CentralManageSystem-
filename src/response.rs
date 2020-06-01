use serde::Serialize;

#[derive(Serialize)]
pub struct ResponseBody<T> {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ResponseBody<T> {
    pub fn new_success(data: Option<T>) -> Self {
        ResponseBody {
            code: 0,
            message: String::from("success"),
            data,
        }
    }

    ///响应错误返回
    pub fn new_json_parse_error() -> Self {
        ResponseBody {
            code: 90001,
            message: String::from("json parse error."),
            data: None,
        }
    }

    ///作为文件类错误相关返回
    pub fn new_file_error() -> Self {
        ResponseBody {
            code: 90002,
            message: String::from("file open or write or read error."),
            data: None,
        }
    }

    ///作为字符转换相关产生的错误返回
    pub fn new_str_conver_error() -> Self {
        ResponseBody {
            code: 90003,
            message: String::from("char conversion error"),
            data: None,
        }
    }

    ///作为数据库操作失败时返回
    pub fn database_build_error() -> Self {
        ResponseBody {
            code: 90004,
            message: String::from("Database operation failed"),
            data: None,
        }
    }
}
