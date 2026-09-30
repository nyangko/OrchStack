//! 공통 에러 응답: 모든 실패를 `{ "error": "<code>", "message": "<msg>" }` JSON으로 돌려준다
use axum::{
    Json,
    extract::{
        FromRequest, FromRequestParts,
        rejection::{JsonRejection, PathRejection},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::{DbErr, SqlErr};
use serde::Serialize;
use utoipa::ToSchema;

/// 핸들러 반환 형태
pub type Res<T> = Result<T, Error>;

/// 에러 응답 본문 (OpenAPI 공통 에러 스키마)
#[derive(Serialize, ToSchema)]
pub struct ErrorBody {
    /// 에러 코드 (예: not_found, invalid_ref)
    error: &'static str,
    /// 사람이 읽는 설명
    message: String,
}

/// HTTP 상태 + 에러 코드 + 설명
#[derive(Debug)]
pub struct Error(StatusCode, &'static str, String);

impl Error {
    /// 대상이 없을 때 (404)
    pub fn not_found() -> Self {
        Self(StatusCode::NOT_FOUND, "not_found", "not found".into())
    }

    /// 값이 허용 범위를 벗어났을 때 (422)
    pub fn invalid(msg: String) -> Self {
        Self(StatusCode::UNPROCESSABLE_ENTITY, "invalid", msg)
    }

    /// 현재 상태에서 할 수 없는 조작일 때 (409)
    pub fn conflict(msg: String) -> Self {
        Self(StatusCode::CONFLICT, "conflict", msg)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.0, Json(ErrorBody { error: self.1, message: self.2 })).into_response()
    }
}

/// DB 오류 변환. 외래키 위반은 잘못된 입력이므로 422, 나머지는 500 (내용은 노출하지 않는다)
impl From<DbErr> for Error {
    fn from(e: DbErr) -> Self {
        match e.sql_err() {
            Some(SqlErr::ForeignKeyConstraintViolation(_)) => {
                Self(StatusCode::UNPROCESSABLE_ENTITY, "invalid_ref", "referenced row does not exist".into())
            }
            _ => {
                eprintln!("db error: {e}");
                Self(StatusCode::INTERNAL_SERVER_ERROR, "internal", "internal error".into())
            }
        }
    }
}

/// 삭제 시 외래키 위반 = 다른 행이 참조 중(RESTRICT) → 409. 나머지는 일반 DB 오류
pub fn in_use(e: DbErr) -> Error {
    // SQLite는 RESTRICT 위반을 FK 코드(787)가 아닌 1811로 보내 sql_err()가 못 잡는다 → 메시지로 판별
    if e.to_string().contains("FOREIGN KEY constraint failed") {
        return Error::conflict("in use by other rows".into());
    }
    e.into()
}

/// 본문 JSON 파싱 실패 변환 (상태 코드는 axum이 정한 400/415/422 그대로)
impl From<JsonRejection> for Error {
    fn from(e: JsonRejection) -> Self {
        Self(e.status(), "bad_body", e.body_text())
    }
}

/// 경로 값 파싱 실패 변환 (예: /projects/abc)
impl From<PathRejection> for Error {
    fn from(e: PathRejection) -> Self {
        Self(e.status(), "bad_path", e.body_text())
    }
}

/// JSON 본문. axum::Json과 같지만 실패를 공통 에러 형식으로 돌려준다
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(Error))]
pub struct Body<T>(pub T);

/// 경로의 번호(sn) 1개. axum::Path와 같지만 실패를 공통 에러 형식으로 돌려준다
#[derive(FromRequestParts, serde::Deserialize)]
#[from_request(via(axum::extract::Path), rejection(Error))]
pub struct Sn(pub i64);
