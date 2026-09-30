//! 명령 실행 틀: 상태 변경 + tbl_log_event append + seq 발급을 한 트랜잭션에서 하고, 커밋한 뒤 broadcast로 내보낸다
use crate::{entity::tbl_log_event as e, error::Res};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, TransactionTrait};
use serde::Serialize;
use serde_json::Value;
use std::sync::LazyLock;
use tokio::sync::{Mutex, broadcast};

// ponytail: 프로세스 전역 락 1개 (Alpha는 단일 프로세스 · 워크스페이스 1개). 워크스페이스가 늘면 워크스페이스별 락으로
static LOCK: Mutex<()> = Mutex::const_new(());
static BUS: LazyLock<broadcast::Sender<e::Model>> = LazyLock::new(|| broadcast::channel(256).0);

/// 발행된 이벤트 구독. 커밋된 이벤트만 온다
#[allow(dead_code)] // B-5 스트림이 쓴다
pub fn subscribe() -> broadcast::Receiver<e::Model> {
    BUS.subscribe()
}

/// 명령 하나가 만드는 이벤트 (seq · 시각 · 행위자는 `run`이 채운다)
pub struct Ev {
    project_sn: Option<i64>,
    agg: &'static str,
    sn: i64,
    kind: &'static str,
    payload: Value,
}

impl Ev {
    /// 대상 종류(agg) · 번호(sn)의 kind 이벤트. payload는 JSON으로 저장된다
    pub fn new(project_sn: Option<i64>, agg: &'static str, sn: i64, kind: &'static str, payload: &impl Serialize) -> Self {
        Self { project_sn, agg, sn, kind, payload: serde_json::to_value(payload).unwrap_or(Value::Null) }
    }
}

/// 쓰기 직렬화 락 → 트랜잭션 → `f`(상태 변경 + 만든 이벤트 목록) → 이벤트 append → 커밋 → 발행.
/// `f`가 실패하면 롤백되어 상태도 이벤트도 남지 않는다
pub async fn run<T>(db: &DatabaseConnection, f: impl AsyncFnOnce(&DatabaseTransaction) -> Res<(T, Vec<Ev>)>) -> Res<T> {
    let _guard = LOCK.lock().await;
    let tx = db.begin().await?;
    let (out, evs) = f(&tx).await?;
    let mut done = Vec::new();
    for (idx, ev) in evs.into_iter().enumerate() {
        // 대상별 순번: 마지막 seq + 1 (락 안이라 경합 없음)
        let seq = e::Entity::find().filter(e::Column::AggregateType.eq(ev.agg)).filter(e::Column::AggregateSn.eq(ev.sn))
            .order_by_desc(e::Column::Seq).one(&tx).await?.map_or(1, |m| m.seq + 1);
        done.push(e::ActiveModel {
            wid: Set(crate::WID),
            project_sn: Set(ev.project_sn),
            aggregate_type: Set(ev.agg.into()),
            aggregate_sn: Set(ev.sn),
            seq: Set(seq),
            event_type: Set(ev.kind.into()),
            payload_json: Set(ev.payload.to_string()),
            command_idx: Set(idx as i64),
            actor_type: Set("user".into()),
            uid: Set(Some(crate::UID)),
            ..Default::default()
        }.insert(&tx).await?);
    }
    tx.commit().await?;
    for m in done {
        let _ = BUS.send(m); // 구독자가 없으면 Err — 무시
    }
    Ok(out)
}
