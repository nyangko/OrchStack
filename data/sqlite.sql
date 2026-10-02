-- =====================================================================
-- OrchStack Alpha · 데이터베이스 스키마 (SQLite · SeaORM)
-- 기준: design/OrchStack Alpha UI.pen (2026-09-29) + README 작업 모델
--
-- 네이밍 규칙
--   tbl_<이름>        기본 테이블 (현재 상태)
--   tbl_map_<a>_<b>   두 테이블을 잇는 연결 테이블
--   tbl_log_<이름>    시간순으로 쌓이기만 하는 기록
--   기본 키           sn (INTEGER AUTOINCREMENT)
--   참조 컬럼         <대상>_sn (사용자 = user_sn · 워크스페이스 = workspace_sn)
--   열거값            status · kind · mode 는 CHECK 제약이 원본 (#9 상태 표 · 프론트 status.ts · 백엔드 전이 표는 이 값을 따른다)
--   토큰 · 비용       모르면 NULL (0으로 위장하지 않는다 · #16) · 금액은 *_usd_micro INTEGER (1달러 = 1,000,000 · REAL 합산 오차 방지)
--   시각              create_at / update_at · UTC 'YYYY-MM-DD HH:MM:SS'
--   참/거짓           is_<이름> · 0 = 아니오, 1 = 예
--
-- 작업 모델
--   Project └ Issue(설계) └ Task(실행 단위) └ Run(실행 시도) └ Session(CLI 세션)
--   Template(기본값) → 복사 → Member(팀의 실제 작업자)
--   Runtime(CLI) × Connection(구독 · 플랜 · API 키 · 게이트웨이 · 로컬)
--
-- 주석: SQLite는 COMMENT 구문이 없어서 컬럼 옆 `--` 주석으로 적는다.
--       (sqlite_master.sql 에 원문 그대로 보존됨)
--
-- 사용 방법 (#11)
--   이 파일이 스키마 원본이다. 서버(backend)가 첫 실행 시 이 파일로 테이블을 만들고
--   기본 사용자(sn=1) · 워크스페이스(sn=1)를 넣는다. SeaORM migration은 쓰지 않는다.
--   Rust entity는 `sea-orm-cli generate entity`로 이 스키마에서 생성한다 (backend/src/entity).
--   Alpha는 SQLite 전용이다 (AUTOINCREMENT · datetime('now') · 식 인덱스 사용). 다른 DB 이식은 범위 밖.
--   상태 값 · 이벤트 구조는 #9 Core Domain Model · #10 Command → Event → Projection 기준.
--   화면 전용 값(진행률 %, 저장된 보기 등)은 domain이 아니라 조회 전용으로 다룬다.
-- =====================================================================

PRAGMA foreign_keys = ON;


-- =====================================================================
-- 1. 사용자 · 워크스페이스
-- =====================================================================

-- 사용자. Alpha는 로컬 1인 사용(sn = 1), 차후 가입 · 로그인 지원
CREATE TABLE tbl_user (
    sn              INTEGER PRIMARY KEY AUTOINCREMENT,              -- 사용자 번호
    email           TEXT UNIQUE,                                    -- 로그인 이메일 (로그인 지원 전에는 NULL)
    password_hash   TEXT,                                           -- 비밀번호 해시 (원문 저장 금지)
    name            TEXT NOT NULL,                                  -- 표시 이름 (화면의 '나')
    initial         TEXT,                                           -- 아바타 이니셜 (예: S)
    ui_language     TEXT,                                           -- 화면 언어 (NULL = 워크스페이스 기본 · 다국어 지원)
    task_view_json  TEXT,                                           -- 저장된 보기 JSON 배열 [{sn, name, filter, sort}] · Tasks 페이지의 필터 묶음 (조회 전용, domain 아님) · filter = 프로젝트 · 담당 · 상태 · 우선순위 · 정렬 조건 객체 · sn = 이 배열 안의 번호
    status          TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','blocked','left')),  -- 상태: active(사용 중) | blocked(로그인 차단) | left(탈퇴 · 기록만 보존)
    last_login_at   TEXT,                                           -- 마지막 로그인 시각
    create_at       TEXT NOT NULL DEFAULT (datetime('now')),        -- 생성 시각
    update_at       TEXT NOT NULL DEFAULT (datetime('now'))         -- 수정 시각
);

-- 워크스페이스. Settings › 일반 · 실행기 공통 설정 · 스킬 설치 정책 · 알림 시간대
CREATE TABLE tbl_workspace (
    sn                       INTEGER PRIMARY KEY AUTOINCREMENT,     -- 워크스페이스 번호
    user_sn                      INTEGER NOT NULL REFERENCES tbl_user(sn) ON DELETE RESTRICT,  -- 소유자
    name                     TEXT NOT NULL,                         -- 워크스페이스 이름 (예: OrchStack)
    default_repo             TEXT,                                  -- 기본 저장소 (예: orchstack/app)
    timezone                 TEXT NOT NULL DEFAULT 'Asia/Seoul',    -- 시간대 (리셋 시각 · 타이머 · 일일 요약 기준)
    ui_language              TEXT NOT NULL DEFAULT 'ko',            -- 화면 언어
    report_language          TEXT NOT NULL DEFAULT 'ko',            -- 에이전트 보고 · 요약 · 질문 언어 (연결 · 멤버 기본값)
    commit_language          TEXT NOT NULL DEFAULT 'en',            -- 커밋 메시지 · PR 본문 · 코드 주석 언어
    date_format              TEXT NOT NULL DEFAULT 'YYYY-MM-DD',    -- 날짜 표시 형식
    theme                    TEXT NOT NULL DEFAULT 'system' CHECK (theme IN ('light','dark','system')),  -- 테마: light(라이트) | dark(다크) | system(OS 설정 따름)
    retention_run_log_day    INTEGER NOT NULL DEFAULT 90,           -- Run 로그 보관 일수
    retention_token_day      INTEGER NOT NULL DEFAULT 365,          -- 토큰 · 비용 기록 보관 일수
    retention_decision_day   INTEGER NOT NULL DEFAULT 0,            -- 결정 기록 보관 일수 (0 = 영구)
    github_mode              TEXT NOT NULL DEFAULT 'bot' CHECK (github_mode IN ('bot','personal')),  -- 커밋 · PR 작성자: bot(전용 계정 · 사람 작업과 구분) | personal(사용자 개인 계정)
    github_account           TEXT,                                  -- GitHub 계정 이름 (예: orch-bot)
    github_repo_scope        TEXT,                                  -- 접근 가능한 저장소 범위 (예: orchstack/*)
    max_concurrent_run       INTEGER NOT NULL DEFAULT 3,            -- 기기 전체 동시 실행 Run 수
    run_timeout_min          INTEGER NOT NULL DEFAULT 20,           -- Run 제한 시간(분) · 넘으면 연장 승인 요청
    workdir_mode             TEXT NOT NULL DEFAULT 'repo' CHECK (workdir_mode IN ('worktree','repo')),  -- 작업 공간 방식: worktree(Run마다 git worktree로 격리) | repo(저장소 폴더에서 직접 작업)
    workdir_root             TEXT,                                  -- 작업 공간 폴더 (예: ~/.orch/worktrees)
    worktree_keep_hour       INTEGER NOT NULL DEFAULT 24,           -- 실패 · 취소 Run의 worktree 보관 시간(조사용) · 지나면 정리. 성공 Run은 병합 직후 정리
    detect_path              TEXT,                                  -- 실행기를 찾을 경로 목록 (콜론 구분)
    is_network_sandbox       INTEGER NOT NULL DEFAULT 1,            -- 허용 도메인만 접근
    is_runtime_auto_update   INTEGER NOT NULL DEFAULT 1,            -- 실행기 패치 버전 자동 업데이트
    is_skill_scan_required   INTEGER NOT NULL DEFAULT 1,            -- 보안 검사 통과한 스킬만 추가
    is_skill_approval        INTEGER NOT NULL DEFAULT 1,            -- 스킬 추가 전 사용자 승인
    is_skill_pin_version     INTEGER NOT NULL DEFAULT 1,            -- 스킬 버전 고정 (sha256)
    is_skill_update_notify   INTEGER NOT NULL DEFAULT 0,            -- 스킬 새 버전 알림
    dnd_start                TEXT,                                  -- 방해 금지 시작 (예: 22:00)
    dnd_end                  TEXT,                                  -- 방해 금지 끝 (예: 08:00)
    is_dnd_weekend           INTEGER NOT NULL DEFAULT 1,            -- 주말 전체 방해 금지
    dnd_bypass_level         INTEGER NOT NULL DEFAULT 3,            -- 이 레벨 이상은 방해 금지 무시 (L3)
    daily_summary_time       TEXT,                                  -- 일일 요약 시각 (예: 18:00)
    channel_json             TEXT,                                  -- 알림 채널 JSON 배열 [{kind, status, target, target_label, key_ref, update_at}] · kind = app(앱 내) | desktop(데스크톱 알림) | telegram | email · status = on(항상 켜짐 · 앱) | allowed(OS 권한 허용됨) | connected(연결됨) | off(설정 안 됨) · key_ref = 봇 토큰의 키체인 항목 이름 (응답에 내보내지 않는다)
    notify_json              TEXT,                                  -- 알림 규칙 기본값 JSON 배열 [{event_code, channel_kind, is_enabled}] · event_code = decision_request | approval_request | orch_decided | run_failed | guard_stop | context_warn | quota_low | budget_80 | budget_over | connection_error | fallback_used | pr | task_done | daily_summary · 행이 없는 이벤트 × 채널은 켜짐
    is_onboarded             INTEGER NOT NULL DEFAULT 0,            -- 인트로(연결 → 프로젝트 → 기본 팀) 완료 여부
    create_at                TEXT NOT NULL DEFAULT (datetime('now')),  -- 생성 시각
    update_at                TEXT NOT NULL DEFAULT (datetime('now'))   -- 수정 시각
);


-- =====================================================================
-- 2. 실행기 · 연결 · 모델
-- =====================================================================

-- 실행기(CLI). 에이전트가 실제로 도는 프로그램 · Settings › 실행기
CREATE TABLE tbl_runtime (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 실행기 번호
    workspace_sn     INTEGER NOT NULL REFERENCES tbl_workspace(sn) ON DELETE CASCADE, -- 워크스페이스
    code             TEXT NOT NULL,                                 -- 식별 코드: codex(Codex CLI) | claude_code(Claude Code) | gemini(Gemini CLI) | opencode(OpenCode) | cursor(Cursor Agent) | kiro(Kiro CLI)
    name             TEXT NOT NULL,                                 -- 표시 이름 (예: Codex CLI)
    version          TEXT,                                          -- 설치된 버전 (예: 0.41)
    latest_version   TEXT,                                          -- 업데이트 가능한 최신 버전
    bin_path         TEXT,                                          -- 실행 파일 경로
    install_status   TEXT NOT NULL DEFAULT 'missing' CHECK (install_status IN ('installed','missing')),  -- 설치 상태: installed(PATH에서 감지됨) | missing(설치 안 됨)
    login_status     TEXT NOT NULL DEFAULT 'none' CHECK (login_status IN ('logged_in','login_required','none')),  -- 로그인 상태: logged_in(로그인됨) | login_required(로그인 필요 · 실행 불가) | none(로그인이 필요 없는 실행기)
    sort             INTEGER NOT NULL DEFAULT 0,                    -- 목록 순서
    detect_at        TEXT,                                          -- 마지막 감지 시각
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    update_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 수정 시각
    UNIQUE (workspace_sn, code)
);

-- 모델 연결. 모델을 부르는 경로 · Settings › 모델 연결 · 연결 추가
CREATE TABLE tbl_connection (
    sn                   INTEGER PRIMARY KEY AUTOINCREMENT,         -- 연결 번호
    workspace_sn         INTEGER NOT NULL REFERENCES tbl_workspace(sn) ON DELETE CASCADE,  -- 워크스페이스
    user_sn                  INTEGER REFERENCES tbl_user(sn) ON DELETE SET NULL,          -- 연결을 추가한 사용자
    kind                 TEXT NOT NULL CHECK (kind IN ('subscription','plan','api_key','gateway','local')),  -- 종류: subscription(구독 · CLI 로그인 · 남은 비율로 한도 표시) | plan(코딩 도구 요금제 · 월 사용량) | api_key(API 키 · 사용한 만큼 과금) | gateway(OpenAI 호환 게이트웨이 · 비용 + 폴백 횟수) | local(로컬 모델 · 한도 없음)
    provider_code        TEXT NOT NULL,                             -- 제공자 코드: anthropic | openai | google | github_copilot | cursor | kimi | zai | omniroute | openrouter | vercel | ollama | lmstudio | openai_compatible(직접 입력) …
    provider_name        TEXT NOT NULL,                             -- 제공자 표시 이름 (예: Anthropic · Max)
    name                 TEXT NOT NULL,                             -- 연결 이름 (예: vercel-team · 멤버 화면에 보임)
    account_label        TEXT,                                      -- 계정 표시 (예: claude-team@orch)
    plan_name            TEXT,                                      -- 요금제 이름 (예: ChatGPT Pro, Max)
    runtime_sn           INTEGER REFERENCES tbl_runtime(sn) ON DELETE SET NULL,        -- 구독 로그인에 쓰는 실행기 (구독은 이 CLI 안에서만 사용)
    login_method         TEXT CHECK (login_method IN ('browser','device_code','terminal')),  -- 구독 로그인 방식: browser(브라우저 로그인) | device_code(기기 코드 입력) | terminal(터미널에서 CLI로 직접)
    base_url             TEXT,                                      -- 엔드포인트 (게이트웨이 · 로컬 · OpenAI 호환)
    key_ref              TEXT,                                      -- 키체인 항목 이름 (키 원문은 DB에 저장하지 않음)
    key_hint             TEXT,                                      -- 키 끝 4자리 (예: 3f9a)
    status               TEXT NOT NULL DEFAULT 'available' CHECK (status IN ('connected','checking','login_required','expired','error','available')),  -- 상태: connected(연결됨 · 사용 가능) | checking(연결 확인 중) | login_required(로그인 필요) | expired(키 · 토큰 만료 · 폴백에서 제외) | error(연결 오류 · 폴백에서 제외) | available(추가 가능 · 아직 연결 안 함)
    status_message       TEXT,                                      -- 상태 설명 (예: 키 만료 · 401)
    monthly_budget_usd_micro INTEGER,                              -- 월 예산 (1달러 = 1,000,000) · API 키 · 게이트웨이
    monthly_fee_usd_micro INTEGER,                                 -- 월 정액 요금 (1달러 = 1,000,000 · NULL = 없음) · 구독 · 요금제 · 월 비용 집계의 subscription_fixed
    budget_warn_percent  INTEGER NOT NULL DEFAULT 80,               -- 예산 경고 기준(%)
    is_budget_exclude    INTEGER NOT NULL DEFAULT 1,                -- 예산 초과 시 폴백에서 자동 제외
    scope                TEXT NOT NULL DEFAULT 'workspace' CHECK (scope IN ('workspace','team','me')),  -- 사용 범위: workspace(워크스페이스 전체) | team(선택한 팀만 · team_json) | me(추가한 사용자만)
    report_language      TEXT,                                      -- 이 연결의 보고 언어 (NULL = 워크스페이스 기본)
    commit_language      TEXT,                                      -- 이 연결의 커밋 · PR 언어 (NULL = 워크스페이스 기본)
    latency_ms           INTEGER,                                   -- 마지막 연결 테스트 응답 시간
    cache_hit_percent    INTEGER,                                   -- 캐시 적중률(%) · 게이트웨이
    test_at              TEXT,                                      -- 마지막 연결 테스트 시각
    sync_at              TEXT,                                      -- 마지막 모델 목록 동기화 시각
    team_json            TEXT,                                      -- 사용 범위가 team일 때 허용 팀 번호 JSON 배열 (예: [1, 3])
    quota_json           TEXT,                                      -- 현재 한도 JSON 배열 [{period, unit, used_value, limit_value, remain_percent, reset_at, update_at}] · period = minute(분당) | 5h(5시간 창) | day(일간) | week(주간) | month(월간) · unit = percent | usd | request | token · limit_value = 한도 (구독처럼 알 수 없으면 NULL) · 추이는 tbl_log_token에서 계산 · 갱신은 실행기가 한다
    notify_json          TEXT,                                      -- 이 연결만의 알림 규칙 JSON 배열 [{event_code, channel_kind, is_enabled}] · 행이 없는 이벤트 × 채널은 워크스페이스 기본(tbl_workspace.notify_json)을 따른다
    create_at            TEXT NOT NULL DEFAULT (datetime('now')),   -- 생성 시각
    update_at            TEXT NOT NULL DEFAULT (datetime('now'))    -- 수정 시각
);


-- 연결에서 쓸 수 있는 모델. 모델 선택 다이얼로그
CREATE TABLE tbl_model (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 모델 번호
    connection_sn    INTEGER NOT NULL REFERENCES tbl_connection(sn) ON DELETE CASCADE,  -- 연결
    code             TEXT NOT NULL,                                 -- 모델 ID (예: gpt-5, claude-sonnet-5)
    vendor           TEXT,                                          -- 만든 곳 (예: OpenAI, Anthropic, DeepSeek)
    tag_json         TEXT,                                          -- 특징 태그 JSON 배열 (예: ["코딩","도구 사용"])
    context_limit    INTEGER,                                       -- OrchStack이 쓰는 컨텍스트 한도(토큰) (예: 128000)
    context_max      INTEGER,                                       -- 모델 최대 컨텍스트(토큰) (예: 400000)
    price_in_usd     REAL,                                          -- 입력 100만 토큰당 가격($)
    price_out_usd    REAL,                                          -- 출력 100만 토큰당 가격($)
    speed            TEXT CHECK (speed IN ('fast','normal','slow')),  -- 속도: fast(빠름) | normal(보통) | slow(느림)
    is_recommended   INTEGER NOT NULL DEFAULT 0,                    -- 추천 표시
    is_tested        INTEGER NOT NULL DEFAULT 0,                    -- OrchStack에서 검증됨(Tested)
    is_pinned        INTEGER NOT NULL DEFAULT 0,                    -- 핀 고정 (Harness 카드에 바로 표시 · 최대 3)
    is_visible       INTEGER NOT NULL DEFAULT 1,                    -- 목록에 표시
    sync_at          TEXT,                                          -- 마지막 동기화 시각
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    UNIQUE (connection_sn, code)
);


-- =====================================================================
-- 3. 스킬 · MCP
-- =====================================================================

-- 스킬 소스. 스킬을 찾고 설치하는 곳 · Settings › 스킬 소스
CREATE TABLE tbl_skill_source (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 소스 번호
    workspace_sn     INTEGER NOT NULL REFERENCES tbl_workspace(sn) ON DELETE CASCADE, -- 워크스페이스
    kind             TEXT NOT NULL CHECK (kind IN ('skills_sh','github','local','marketplace','builtin')),  -- 종류: skills_sh(skills.sh 공개 디렉터리) | github(GitHub 저장소) | local(로컬 폴더) | marketplace(마켓플레이스) | builtin(기본 내장)
    name             TEXT NOT NULL,                                 -- 표시 이름 (예: orchstack/team-skills)
    location         TEXT,                                          -- 주소 또는 폴더 경로
    branch           TEXT,                                          -- 브랜치 (GitHub 소스)
    status           TEXT NOT NULL DEFAULT 'connected' CHECK (status IN ('connected','error','off')),  -- 상태: connected(연결됨 · 동기화 중) | error(동기화 실패) | off(사용 안 함)
    sort             INTEGER NOT NULL DEFAULT 0,                    -- 우선순위 (위에서부터 우선)
    sync_at          TEXT,                                          -- 마지막 동기화 시각
    create_at        TEXT NOT NULL DEFAULT (datetime('now'))        -- 생성 시각
);

-- 스킬. 워크스페이스에 설치된 스킬 · Skills & MCP
CREATE TABLE tbl_skill (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 스킬 번호
    workspace_sn     INTEGER NOT NULL REFERENCES tbl_workspace(sn) ON DELETE CASCADE, -- 워크스페이스
    source_sn        INTEGER REFERENCES tbl_skill_source(sn) ON DELETE SET NULL,       -- 가져온 소스
    name             TEXT NOT NULL,                                 -- 스킬 이름 (예: svelte-ui)
    description      TEXT,                                          -- 설명
    version          TEXT,                                          -- 설치된 버전
    latest_version   TEXT,                                          -- 업데이트 가능한 버전 (없으면 NULL)
    sha256           TEXT,                                          -- 버전 고정용 해시
    token_cost       INTEGER NOT NULL DEFAULT 0,                    -- Run 컨텍스트에 더해지는 토큰 수 (예: 420)
    scan_status      TEXT NOT NULL DEFAULT 'pending' CHECK (scan_status IN ('pending','passed','failed')),  -- 보안 검사: pending(검사 전 · 추가 불가) | passed(통과) | failed(실패 · 차단)
    scan_message     TEXT,                                          -- 검사 실패 사유 (예: 외부 URL 실행)
    is_enabled       INTEGER NOT NULL DEFAULT 1,                    -- 워크스페이스에서 허용 (끄면 모든 멤버에서 빠짐)
    is_blocked       INTEGER NOT NULL DEFAULT 0,                    -- 차단됨
    install_at       TEXT NOT NULL DEFAULT (datetime('now')),       -- 설치 시각
    update_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 수정 시각
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    UNIQUE (workspace_sn, name)
);

-- MCP 서버. 에이전트에게 도구를 주는 서버 · Tools & MCP
CREATE TABLE tbl_mcp (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- MCP 번호
    workspace_sn     INTEGER NOT NULL REFERENCES tbl_workspace(sn) ON DELETE CASCADE, -- 워크스페이스
    name             TEXT NOT NULL,                                 -- 서버 이름 (예: playwright-mcp)
    description      TEXT,                                          -- 설명
    tool_count       INTEGER NOT NULL DEFAULT 0,                    -- 제공 도구 수
    token_cost       INTEGER NOT NULL DEFAULT 0,                    -- 설치 시 Run 컨텍스트에 더해지는 토큰 수
    install_status   TEXT NOT NULL DEFAULT 'available' CHECK (install_status IN ('installed','available')),  -- 설치 상태: installed(설치됨 · 매 Run 컨텍스트에 도구 추가) | available(접근 가능 · 필요할 때 설치)
    is_auth_required INTEGER NOT NULL DEFAULT 0,                    -- 인증 필요 여부
    auth_status      TEXT NOT NULL DEFAULT 'none' CHECK (auth_status IN ('ok','required','none')),  -- 인증 상태: ok(인증됨) | required(인증 필요 · 사용 불가) | none(인증이 필요 없음)
    health           TEXT NOT NULL DEFAULT 'unknown' CHECK (health IN ('ok','error','unknown')),  -- 점검 결과: ok(정상) | error(오류) | unknown(점검 전)
    config_json      TEXT,                                          -- 실행 설정 JSON (command, args, env 이름)
    check_at         TEXT,                                          -- 마지막 점검 시각
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    update_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 수정 시각
    UNIQUE (workspace_sn, name)
);


-- =====================================================================
-- 4. 에이전트 프로필 · 템플릿 · 팀 · 멤버
--    프로필 = 하네스 + 지침 + 스킬 + 도구 + 권한 묶음.
--    템플릿 버전 · 멤버 · 워크스페이스 기본값이 각각 프로필 1개를 가진다.
--    멤버 추가 = 템플릿 프로필을 복사해서 새 프로필을 만든다.
-- =====================================================================

-- 에이전트 프로필. Harness · 권한 · 한도 · 도구 · 규칙 · 가드 · 폴백 (작은 목록은 *_json 컬럼에 통째로 둔다)
CREATE TABLE tbl_agent_profile (
    sn                    INTEGER PRIMARY KEY AUTOINCREMENT,        -- 프로필 번호
    workspace_sn          INTEGER NOT NULL REFERENCES tbl_workspace(sn) ON DELETE CASCADE,  -- 워크스페이스
    kind                  TEXT NOT NULL CHECK (kind IN ('workspace','template','member')),  -- 소유 종류: workspace(워크스페이스 기본값) | template(템플릿 버전) | member(멤버 · 템플릿 복사본)
    runtime_sn            INTEGER REFERENCES tbl_runtime(sn) ON DELETE SET NULL,       -- 실행기
    connection_sn         INTEGER REFERENCES tbl_connection(sn) ON DELETE SET NULL,    -- 연결
    model_sn              INTEGER REFERENCES tbl_model(sn) ON DELETE SET NULL,         -- 모델
    effort                TEXT NOT NULL DEFAULT 'auto' CHECK (effort IN ('auto','low','medium','high')),  -- 추론 강도: auto(모델 기본) | low(낮음) | medium(중간) | high(높음)
    session_mode          TEXT NOT NULL DEFAULT 'resume_task' CHECK (session_mode IN ('resume_task','new_run')),  -- 세션 방식: resume_task(같은 태스크면 세션 이어가기) | new_run(Run마다 새 세션)
    repo_rule_mode        TEXT NOT NULL DEFAULT 'use' CHECK (repo_rule_mode IN ('use','ignore')),  -- 저장소 규칙(AGENTS.md · CLAUDE.md · GEMINI.md): use(그대로 사용 · 겹치는 프리셋 규칙은 경고) | ignore(무시) 
    workdir               TEXT,                                     -- 작업 폴더 (예: web-platform/frontend)
    trust_level           INTEGER NOT NULL DEFAULT 3 CHECK (trust_level BETWEEN 1 AND 4),  -- Trust 레벨: 1(읽기 전용 · 코드 읽기 · 분석만) | 2(제안만 · diff 제출, 사람이 적용) | 3(워크스페이스 쓰기 · 허용 범위 안에서 수정 · 커밋) | 4(자율 · push · PR까지 자동, 리뷰 필수)
    github_mode           TEXT CHECK (github_mode IN ('bot','personal')),  -- 커밋 작성자: bot(전용 계정) | personal(사용자 개인 계정) · NULL = 워크스페이스 기본
    network_mode          TEXT NOT NULL DEFAULT 'allowlist' CHECK (network_mode IN ('allowlist','open','off')),  -- 네트워크: allowlist(허용 도메인만) | open(제한 없음) | off(차단)
    run_token_limit       INTEGER,                                  -- Run 하나의 토큰 한도 (예: 120000)
    context_warn_percent  INTEGER NOT NULL DEFAULT 80,              -- 컨텍스트 경고 기준(%)
    run_time_limit_min    INTEGER,                                  -- Run 시간 한도(분) (예: 120)
    auto_retry_max        INTEGER NOT NULL DEFAULT 1,               -- 실패 시 자동 재시도 횟수
    path_json             TEXT,                                     -- 파일 범위 JSON 배열 [{kind, pattern}] · kind = include(포함) | exclude(제외 · 포함보다 우선) · pattern = glob (예: frontend/**, **/.env*)
    tool_json             TEXT,                                     -- CLI 기본 도구 정책 JSON 배열 [{tool_code, scope_text, policy}] · tool_code = read(파일 읽기) | edit(파일 수정 · 쓰기) | shell(명령 실행) | git_push(push · PR) | web_fetch(웹 요청) | git_destructive(force push · reset --hard) · policy = allow(허용) | allowlist(허용 목록만) | approval(승인 필요) | block(항상 차단) · 행이 없는 도구는 allow
    rule_json             TEXT,                                     -- 승인 규칙 · 항상 차단 JSON 배열 [{action_code, title, pattern, description, policy, approver, is_notify}] · action_code = pr_create | dependency_add | external_message | env_access | run_extend | command(명령 패턴 · pattern = 명령 글자) · policy = auto(자동 허용) | approval(승인 필요 · tbl_ask 승인 요청 생성) | block(항상 차단) · approver = user | orch_then_user
    guard_json            TEXT,                                     -- 가드 트리거 JSON 배열 [{name, stage, pattern, is_enabled}] · Run 도중 위험 신호를 감지하면 멈추고 알림 · stage = tool_use(도구 실행 직전) | tool_result(도구 결과) | output(모델 출력)
    fallback_json         TEXT,                                     -- 폴백 체인 JSON 배열 (위에서부터 시도) [{runtime_sn, connection_sn, model_sn, switch_rule, max_level, tier}] · model_sn NULL = 연결 기본 · switch_rule = 다음으로 넘어가는 조건 (예: 429) · max_level = 맡을 수 있는 최대 작업 레벨 · tier = S | M | L 하위 작업 모델 등급 (NULL = 모든 등급 · #67) · 연결을 지우려면 먼저 체인에서 빼야 한다 (앱에서 409)
    create_at             TEXT NOT NULL DEFAULT (datetime('now')),  -- 생성 시각
    update_at             TEXT NOT NULL DEFAULT (datetime('now'))   -- 수정 시각
);

-- 개별 지침. 프리셋에 없는 템플릿 · 멤버 전용 추가 규칙 (보통 1개 · 예: rules/accessibility.md) · 프리셋 조합 뒤에 붙는다
CREATE TABLE tbl_profile_file (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 파일 번호
    profile_sn       INTEGER NOT NULL REFERENCES tbl_agent_profile(sn) ON DELETE CASCADE,  -- 프로필
    path             TEXT NOT NULL,                                 -- 파일 경로 (예: rules/accessibility.md)
    content          TEXT NOT NULL DEFAULT '',                      -- 내용 (Markdown)
    token_count      INTEGER NOT NULL DEFAULT 0,                    -- 토큰 수
    sort             INTEGER NOT NULL DEFAULT 0,                    -- 탭 순서
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    update_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 수정 시각
    UNIQUE (profile_sn, path)
);

-- 프로필 ↔ 스킬. 이 에이전트에서 켠 스킬
CREATE TABLE tbl_map_profile_skill (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 연결 관계 번호
    profile_sn       INTEGER NOT NULL REFERENCES tbl_agent_profile(sn) ON DELETE CASCADE,  -- 프로필
    skill_sn         INTEGER NOT NULL REFERENCES tbl_skill(sn) ON DELETE CASCADE,     -- 스킬
    is_enabled       INTEGER NOT NULL DEFAULT 1,                    -- 활성 여부
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    UNIQUE (profile_sn, skill_sn)
);

-- 프로필 ↔ MCP. 이 에이전트가 쓰는 MCP 서버
CREATE TABLE tbl_map_profile_mcp (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 연결 관계 번호
    profile_sn       INTEGER NOT NULL REFERENCES tbl_agent_profile(sn) ON DELETE CASCADE,  -- 프로필
    mcp_sn           INTEGER NOT NULL REFERENCES tbl_mcp(sn) ON DELETE CASCADE,       -- MCP 서버
    access_mode      TEXT NOT NULL DEFAULT 'accessible' CHECK (access_mode IN ('installed','accessible')),  -- 사용 방식: installed(설치 · 매 Run 컨텍스트에 도구 추가) | accessible(접근 가능 · 허용만, 비용 없음)
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    UNIQUE (profile_sn, mcp_sn)
);


-- 템플릿. 역할별 에이전트 기본값 · Agents 탭
CREATE TABLE tbl_template (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 템플릿 번호
    workspace_sn     INTEGER NOT NULL REFERENCES tbl_workspace(sn) ON DELETE CASCADE, -- 워크스페이스
    user_sn              INTEGER REFERENCES tbl_user(sn) ON DELETE SET NULL,              -- 만든 사용자
    name             TEXT NOT NULL,                                 -- 이름 (예: Frontend Developer)
    role_name        TEXT,                                          -- 역할 설명 (예: 프론트엔드 개발)
    category         TEXT NOT NULL DEFAULT 'dev' CHECK (category IN ('dev','verify','design_pm','orch')),  -- 분류: dev(개발) | verify(검증 · QA · 리뷰) | design_pm(디자인 · PM) | orch(Orch · 모든 팀)
    icon             TEXT,                                          -- 아이콘 이름 (예: monitor)
    color            TEXT,                                          -- 역할 색 토큰 (예: role-frontend)
    description      TEXT,                                          -- 설명
    tag_json         TEXT,                                          -- 태그 JSON 배열 (예: ["frontend","svelte"])
    is_orch          INTEGER NOT NULL DEFAULT 0,                    -- Orch(PM) 템플릿 여부
    status           TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('draft','active','archived')),  -- 상태: draft(초안 · 팀에 추가 불가) | active(사용 중) | archived(보관 · 목록에서 숨김)
    sort             INTEGER NOT NULL DEFAULT 0,                    -- 목록 순서
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    update_at        TEXT NOT NULL DEFAULT (datetime('now'))        -- 수정 시각
);

-- 템플릿 버전. Revisions · 버전마다 프로필 1개
CREATE TABLE tbl_template_revision (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 버전 기록 번호
    template_sn      INTEGER NOT NULL REFERENCES tbl_template(sn) ON DELETE CASCADE,  -- 템플릿
    profile_sn       INTEGER NOT NULL REFERENCES tbl_agent_profile(sn) ON DELETE CASCADE,  -- 이 버전의 프로필
    version          INTEGER NOT NULL,                              -- 버전 (v3 → 3)
    status           TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','live','archived')),  -- 상태: draft(초안 · 편집 중) | live(배포 중 · 새 멤버가 이 버전을 복사) | archived(지난 버전)
    note             TEXT,                                          -- 변경 요약
    author_type      TEXT NOT NULL DEFAULT 'user' CHECK (author_type IN ('user','orch')),  -- 작성자 종류: user(사용자) | orch(Orch 제안 채택)
    user_sn              INTEGER REFERENCES tbl_user(sn) ON DELETE SET NULL,              -- 작성한 사용자
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    publish_at       TEXT,                                          -- 게시 시각
    UNIQUE (template_sn, version)
);

-- 팀. Teams 탭 · 팀 정책 · Orch 진행 정책(모드 · 타이머 · 레벨 · 가드)
CREATE TABLE tbl_team (
    sn                    INTEGER PRIMARY KEY AUTOINCREMENT,        -- 팀 번호
    workspace_sn          INTEGER NOT NULL REFERENCES tbl_workspace(sn) ON DELETE CASCADE,  -- 워크스페이스
    name                  TEXT NOT NULL,                            -- 팀 이름 (예: Core Team)
    kind                  TEXT NOT NULL DEFAULT 'project' CHECK (kind IN ('orch','project')),  -- 종류: orch(오케스트레이터 · 모든 프로젝트 PM) | project(프로젝트 팀)
    daily_token_budget    INTEGER,                                  -- 하루 토큰 예산 (예: 200000)
    context_warn_percent  INTEGER NOT NULL DEFAULT 80,              -- 컨텍스트 경고 기준(%)
    max_concurrent_run    INTEGER NOT NULL DEFAULT 3,               -- 팀 동시 실행 Run 수
    spawn_mode            TEXT NOT NULL DEFAULT 'runner' CHECK (spawn_mode IN ('sub','fork','runner')),  -- 하위 작업 기본 방식: sub | fork | runner (#67)
    spawn_allow           TEXT NOT NULL DEFAULT 'sub,runner',       -- 허용 방식 (쉼표 구분) · fork는 기본 제외
    max_child_run         INTEGER NOT NULL DEFAULT 3,               -- 리드 Run 1개당 동시 하위 작업 수
    is_review_required    INTEGER NOT NULL DEFAULT 1,               -- 리뷰 필수 여부
    review_stage          TEXT NOT NULL DEFAULT 'before_merge' CHECK (review_stage IN ('before_merge','before_done')),  -- 리뷰 시점: before_merge(PR 병합 전) | before_done(태스크 완료 처리 전)
    repo_scope            TEXT,                                     -- 저장소 권한 범위 (예: orchstack/*)
    repo_permission       TEXT NOT NULL DEFAULT 'branch' CHECK (repo_permission IN ('read','branch','push')),  -- 저장소 권한 수준: read(읽기) | branch(브랜치 생성 · push) | push(기본 브랜치 push 포함)
    orch_mode             TEXT NOT NULL DEFAULT 'auto' CHECK (orch_mode IN ('manual','auto','full_auto')),  -- Orch 진행 모드: manual(매번 사용자 확인) | auto(제안 후 타이머 · 개입 없으면 진행) | full_auto(대기 없이 바로 진행 · 가드는 항상 적용) · 프로젝트가 따로 정하면 그 값이 이긴다 (tbl_project.orch_mode)
    timer_sec             INTEGER NOT NULL DEFAULT 5,               -- 자동 진행 전 대기(초): 3 | 5 | 10 | 30 | 그 밖의 값 = 직접 입력
    is_pause_on_view      INTEGER NOT NULL DEFAULT 1,               -- 사용자가 카드를 보고 있으면 타이머 멈춤
    level_json            TEXT,                                     -- 작업 레벨별 처리 JSON 배열 L0 ~ L4 [{level, name, example, handle, wait_min, no_reply, is_locked}] · handle = auto(바로 자동 진행) | timer(타이머 후 진행) | wait(사용자 응답 대기) | block(항상 차단 · L4 고정) · no_reply = proceed(그대로 진행) | orch_decide(Orch가 근거를 남기고 대신 결정) | keep_wait(계속 대기) | none · NULL = 기본값(앱)
    guard_json            TEXT,                                     -- 루프 가드 JSON 배열 [{code, name, threshold, threshold_unit, scope, sub_threshold, on_trigger, is_enabled, trigger_at}] · code = auto_streak(연속 자동 진행) | reject_loop(반려 → 재작업 반복) | same_failure(같은 실패 반복) | issue_budget(이슈 토큰 · 시간 예산) | orch_new_task(Orch가 만든 새 태스크 수) | user_absent(사용자 부재 감지) · threshold_unit = count | token | minute · scope = issue | task | team · on_trigger = stop(자동 진행 멈춤 · 알림) | to_manual(Manual 모드로 전환) · trigger_at = 마지막으로 걸린 시각 · NULL = 기본값(앱)
    sort                  INTEGER NOT NULL DEFAULT 0,               -- 목록 순서
    create_at             TEXT NOT NULL DEFAULT (datetime('now')),  -- 생성 시각
    update_at             TEXT NOT NULL DEFAULT (datetime('now'))   -- 수정 시각
);

-- 멤버. 팀에서 일하는 실제 에이전트 · 템플릿의 복사본
CREATE TABLE tbl_member (
    sn                INTEGER PRIMARY KEY AUTOINCREMENT,            -- 멤버 번호
    team_sn           INTEGER NOT NULL REFERENCES tbl_team(sn) ON DELETE CASCADE,     -- 소속 팀
    profile_sn        INTEGER NOT NULL REFERENCES tbl_agent_profile(sn) ON DELETE RESTRICT,  -- 멤버 프로필
    template_sn       INTEGER REFERENCES tbl_template(sn) ON DELETE SET NULL,          -- 원본 템플릿 (빈 캐릭터로 시작하면 NULL)
    template_version  INTEGER,                                      -- 복사한 시점의 템플릿 버전
    name              TEXT NOT NULL,                                -- 이름 (예: 진)
    role_name         TEXT NOT NULL,                                -- 역할 (예: Frontend Developer)
    icon              TEXT,                                         -- 아바타 아이콘
    color             TEXT,                                         -- 역할 색 토큰
    is_orch           INTEGER NOT NULL DEFAULT 0,                   -- Orch(PM) 여부
    status            TEXT NOT NULL DEFAULT 'idle' CHECK (status IN ('running','waiting','idle','paused','archived')),  -- 상태 (projection · Run 이벤트 핸들러가 갱신 · 직접 UPDATE 금지): running(Run 실행 중) | waiting(의존 · 판단 · 승인 대기) | idle(할 일 없음) | paused(사용자가 멈춤) | archived(보관 · 삭제 대신 · 기록 유지, Run이 있으면 삭제 불가)
    first_task_mode   TEXT NOT NULL DEFAULT 'orch' CHECK (first_task_mode IN ('orch','task','wait')),  -- 추가 직후: orch(Orch에게 맡김 · 팀 진행 정책대로 배정) | task(지정한 태스크로 시작) | wait(추가만 하고 대기)
    sort              INTEGER NOT NULL DEFAULT 0,                   -- 목록 순서
    create_at         TEXT NOT NULL DEFAULT (datetime('now')),      -- 생성 시각
    update_at         TEXT NOT NULL DEFAULT (datetime('now'))       -- 수정 시각
);


-- =====================================================================
-- 5. 프로젝트 · 이슈 · 태스크
-- =====================================================================

-- 프로젝트. 상단 프로젝트 탭 · 저장소 1개 · Orch 진행 정책을 따로 정하면 그 값이 팀 값을 덮는다
CREATE TABLE tbl_project (
    sn                 INTEGER PRIMARY KEY AUTOINCREMENT,           -- 프로젝트 번호
    workspace_sn       INTEGER NOT NULL REFERENCES tbl_workspace(sn) ON DELETE CASCADE,  -- 워크스페이스
    team_sn            INTEGER REFERENCES tbl_team(sn) ON DELETE SET NULL,             -- 맡은 팀
    name               TEXT NOT NULL,                               -- 이름 (예: OrchStack)
    repo_name          TEXT,                                        -- GitHub 저장소 (예: orchstack/app)
    repo_path          TEXT,                                        -- 로컬 저장소 경로
    default_branch     TEXT NOT NULL DEFAULT 'main',                -- 기본 브랜치
    next_num           INTEGER NOT NULL DEFAULT 1,                  -- 다음 이슈 · 태스크 표시 번호 (둘이 함께 쓰는 번호 발급기)
    is_github_import   INTEGER NOT NULL DEFAULT 0,                  -- GitHub 이슈 가져오기 사용
    import_label       TEXT,                                        -- 가져올 라벨 (예: bug,feature)
    orch_mode          TEXT CHECK (orch_mode IN ('manual','auto','full_auto')),  -- Orch 진행 모드 (NULL = 팀 값) · 값은 tbl_team.orch_mode 와 같다
    timer_sec          INTEGER,                                     -- 자동 진행 전 대기(초) (NULL = 팀 값)
    is_pause_on_view   INTEGER,                                     -- 카드를 보고 있으면 타이머 멈춤 (NULL = 팀 값)
    level_json         TEXT,                                        -- 작업 레벨별 처리 JSON (NULL = 팀 값 · 구조는 tbl_team.level_json)
    guard_json         TEXT,                                        -- 루프 가드 JSON (NULL = 팀 값 · 구조는 tbl_team.guard_json)
    label_json         TEXT,                                        -- 프로젝트 라벨 JSON 배열 [{name, color}] · color = 색 토큰 · 태스크에 처음 붙는 이름은 자동으로 들어간다
    status             TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','archived')),  -- 상태: active(진행 중) | archived(보관 · 읽기 전용)
    sort               INTEGER NOT NULL DEFAULT 0,                  -- 탭 순서
    create_at          TEXT NOT NULL DEFAULT (datetime('now')),     -- 생성 시각
    update_at          TEXT NOT NULL DEFAULT (datetime('now'))      -- 수정 시각
);

-- (Instruction preset · 보고서 양식: 프로젝트 범위 프리셋이 tbl_project 를 참조하므로 프로젝트 다음에 둔다)
-- Instruction preset. 모델 컨텍스트에 들어가는 재사용 지침 (Settings › Instruction presets에서 추가 · 수정 · 삭제)
--   종류별로 역할 템플릿 · 멤버 프로필에 연결만 한다 (tbl_map_profile_preset). 도구 · 권한 정책은 여기 두지 않는다
CREATE TABLE tbl_instruction_preset (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 프리셋 번호
    workspace_sn              INTEGER NOT NULL REFERENCES tbl_workspace(sn) ON DELETE CASCADE,  -- 워크스페이스
    project_sn       INTEGER REFERENCES tbl_project(sn) ON DELETE CASCADE,  -- 프로젝트 전용이면 지정 (NULL = 워크스페이스 전체)
    kind             TEXT NOT NULL CHECK (kind IN ('protocol','role','style','rule','report')),  -- 종류: protocol(내부 블록 규격 · 시스템 소유) | role(역할 · 책임 · 범위) | style(말투 · 출력 태도) | rule(공통 규칙 · 여러 개 연결) | report(태스크 보고서 사람 칸 작성법)
    preset_key       TEXT NOT NULL,                                 -- 식별 키 (예: frontend, token-economy)
    name             TEXT NOT NULL,                                 -- 표시 이름
    description      TEXT,                                          -- 설명 (사람용 · 컨텍스트에 들어가지 않음)
    version          INTEGER NOT NULL DEFAULT 1,                    -- 버전 (행 하나 = 버전 하나 · 수정할 때마다 새 행 · 같은 (kind, preset_key)의 다른 행이 다른 버전)
    is_latest        INTEGER NOT NULL DEFAULT 1,                    -- 그 (kind, preset_key)의 최신 버전인지 (새 버전을 저장하면 이전 행은 0)
    content          TEXT NOT NULL,                                 -- 본문 (Markdown · 컨텍스트에 그대로 들어감 · 허용 변수 {{…}}만 치환)
    token_count      INTEGER NOT NULL DEFAULT 0,                    -- 본문 토큰 수 (저장 시 측정)
    language         TEXT NOT NULL DEFAULT 'en',                    -- 본문 언어 (en 권장 · 토큰 절약)
    source           TEXT NOT NULL DEFAULT 'user' CHECK (source IN ('builtin','user','import','translated')),  -- 출처: builtin(기본 제공 · 앱 배포) | user(사용자 작성) | import(.md · 저장소 AGENTS.md 가져오기) | translated(저장 시 영어 변환)
    change_note      TEXT,                                          -- 변경 요약 (사람용)
    limit_tok        INTEGER NOT NULL,                              -- 토큰 상한 (초과 시 저장 차단)
    is_builtin       INTEGER NOT NULL DEFAULT 0,                    -- 기본 제공 (삭제 불가 · 수정하려면 복제)
    is_locked        INTEGER NOT NULL DEFAULT 0,                    -- 잠김 (protocol · 사용자 수정 불가, 앱 업데이트로만 변경)
    is_default       INTEGER NOT NULL DEFAULT 0,                    -- 새 역할 템플릿에 기본으로 연결
    copy_from_sn     INTEGER REFERENCES tbl_instruction_preset(sn) ON DELETE SET NULL,  -- 복제 원본
    status           TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active','archived')),  -- 상태: active(사용 가능) | archived(보관 · 새로 연결 불가, 기존 연결은 유지)
    user_sn              INTEGER REFERENCES tbl_user(sn) ON DELETE SET NULL,              -- 만든 사용자 (기본 제공이면 NULL)
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    update_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 수정 시각
    UNIQUE (workspace_sn, kind, preset_key, version)
);


-- 프로필 ↔ Instruction preset. 역할 템플릿 · 멤버가 쓰는 프리셋과 고정 버전
--   role · style · report 는 프로필당 1개, rule 은 여러 개 (앱에서 검증) · protocol 은 항상 자동 연결
CREATE TABLE tbl_map_profile_preset (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 연결 관계 번호
    profile_sn       INTEGER NOT NULL REFERENCES tbl_agent_profile(sn) ON DELETE CASCADE,  -- 프로필 (템플릿 버전 · 멤버)
    preset_sn        INTEGER NOT NULL REFERENCES tbl_instruction_preset(sn) ON DELETE RESTRICT,  -- 고정한 프리셋 버전 행 (새 버전은 '업데이트 가능'으로 표시, 사용자가 가져올 때만 이 번호를 바꾼다 · 사용 중이면 삭제 차단 → 보관만 가능)
    sort             INTEGER NOT NULL DEFAULT 0,                    -- 조합 순서 (같은 종류 안에서)
    is_enabled       INTEGER NOT NULL DEFAULT 1,                    -- 사용 여부
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    UNIQUE (profile_sn, preset_sn)
);

-- 보고서 양식. 시스템이 조립하는 화면 양식 (모델 호출 없음 · 0 tok) · Settings › 보고서 양식
CREATE TABLE tbl_report_form (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 양식 번호
    workspace_sn              INTEGER NOT NULL REFERENCES tbl_workspace(sn) ON DELETE CASCADE,  -- 워크스페이스
    kind             TEXT NOT NULL CHECK (kind IN ('task_report','issue_report','pr_body','daily_summary')),  -- 종류: task_report(태스크 보고서) | issue_report(이슈 보고서 · 태스크 합산) | pr_body(PR 본문) | daily_summary(일일 요약 알림)
    form_key         TEXT NOT NULL,                                 -- 식별 키
    name             TEXT NOT NULL,                                 -- 표시 이름
    version          INTEGER NOT NULL DEFAULT 1,                    -- 버전
    body             TEXT NOT NULL,                                 -- 양식 본문 ({{시스템 값}} · [[Agent 사람 칸]] 자리표시)
    locked_json      TEXT,                                          -- 잠긴 칸 JSON 배열 (예: ["unverified","tests.passed_summary"] · 삭제 시 저장 차단)
    is_builtin       INTEGER NOT NULL DEFAULT 0,                    -- 기본 제공
    is_default       INTEGER NOT NULL DEFAULT 0,                    -- 이 종류의 기본 양식
    update_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 수정 시각
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    UNIQUE (workspace_sn, kind, form_key)
);

-- 이슈. 설계 · 명세 단위 · 하위 이슈 가능
CREATE TABLE tbl_issue (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 이슈 번호 (내부)
    project_sn       INTEGER NOT NULL REFERENCES tbl_project(sn) ON DELETE CASCADE,   -- 프로젝트
    parent_sn        INTEGER REFERENCES tbl_issue(sn) ON DELETE SET NULL,              -- 상위 이슈 (하위 이슈일 때)
    num              INTEGER NOT NULL,                              -- 화면 표시 번호 (#51) · tbl_project.next_num 에서 발급
    title            TEXT NOT NULL,                                 -- 제목
    body             TEXT,                                          -- 본문 (Markdown)
    status           TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open','in_progress','done','closed')),  -- 상태 (open · closed는 사용자 · in_progress · done은 태스크 상태에서 계산하는 projection · #9): open(열림 · 시작 전) | in_progress(하위 태스크 진행 중) | done(모든 태스크 완료) | closed(닫힘 · 완료 또는 취소)
    source           TEXT NOT NULL DEFAULT 'manual' CHECK (source IN ('manual','github','orch')),  -- 만든 경로: manual(사용자 작성) | github(GitHub 이슈 가져오기) | orch(Orch 작업 제안에서 생성)
    github_number    INTEGER,                                       -- 연결된 GitHub 이슈 번호
    github_url       TEXT,                                          -- GitHub 이슈 주소
    label_json       TEXT,                                          -- 라벨 이름 JSON 배열 (Issue Board 라벨 · GitHub 이슈 라벨 가져오기)
    user_sn              INTEGER REFERENCES tbl_user(sn) ON DELETE SET NULL,              -- 만든 사용자 (Orch가 만들면 NULL)
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    update_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 수정 시각
    close_at         TEXT,                                          -- 닫은 시각
    UNIQUE (project_sn, num)
);

-- 태스크. 실행 단위 · 칸반 카드
CREATE TABLE tbl_task (
    sn                INTEGER PRIMARY KEY AUTOINCREMENT,            -- 태스크 번호 (내부)
    project_sn        INTEGER NOT NULL REFERENCES tbl_project(sn) ON DELETE CASCADE,  -- 프로젝트
    issue_sn          INTEGER REFERENCES tbl_issue(sn) ON DELETE SET NULL,             -- 속한 이슈
    member_sn         INTEGER REFERENCES tbl_member(sn) ON DELETE SET NULL,            -- 담당 멤버
    num               INTEGER NOT NULL,                             -- 화면 표시 번호 (#129) · tbl_project.next_num 에서 발급
    title             TEXT NOT NULL,                                -- 제목
    description       TEXT,                                         -- 설명 (Markdown)
    status            TEXT NOT NULL DEFAULT 'todo' CHECK (status IN ('backlog','todo','in_progress','blocked','review','done','failed','cancelled')),  -- 상태 (#9): backlog(백로그 · 계획 전) | todo(할 일 · 대기열) | in_progress(진행 중 · Run 실행) | blocked(막힘 · 사람 조치 필요) | review(리뷰 대기 · 리뷰 중) | done(완료) | failed(실패 · 재시도 한도 초과) | cancelled(취소) · waiting(의존 대기)은 저장하지 않고 의존 관계로 계산
    priority          INTEGER NOT NULL DEFAULT 2 CHECK (priority BETWEEN 0 AND 3),  -- 우선순위: 0(P0) ~ 3(P3)
    spawn_mode        TEXT CHECK (spawn_mode IN ('sub','fork','runner')),  -- 하위 작업 방식: sub | fork | runner · NULL = 팀 기본값(tbl_team.spawn_mode) (#67)
    assign_by         TEXT CHECK (assign_by IN ('orch_auto','orch_move','user')),  -- 배정한 쪽: orch_auto(Orch 자동 배정) | orch_move(Orch 재배치) | user(사용자 수동)
    queue_sort        INTEGER,                                      -- 담당 멤버의 실행 대기열 순서
    label_json        TEXT,                                         -- 라벨 이름 JSON 배열 (예: ["auth", "ui"] · 이름순 · 중복 없음) · 프로젝트 라벨 목록은 tbl_project.label_json
    estimate_min      INTEGER,                                      -- 예상 소요(분)
    eta_at            TEXT,                                         -- 예상 완료 시각
    block_reason      TEXT,                                         -- 막힌 이유 (예: 의존성: Redis 설정 대기)
    branch            TEXT,                                         -- 작업 브랜치 (예: feat/login-ui)
    commit_count      INTEGER NOT NULL DEFAULT 0,                   -- 커밋 수
    pr_number         INTEGER,                                      -- PR 번호
    pr_status         TEXT CHECK (pr_status IN ('draft','open','merged','closed')),  -- PR 상태: draft(초안) | open(열림 · 리뷰 대기) | merged(병합) | closed(병합 없이 닫힘)
    create_by         TEXT NOT NULL DEFAULT 'user' CHECK (create_by IN ('user','orch')),  -- 만든 쪽: user(사용자) | orch(Orch)
    user_sn               INTEGER REFERENCES tbl_user(sn) ON DELETE SET NULL,             -- 만든 사용자
    create_at         TEXT NOT NULL DEFAULT (datetime('now')),      -- 생성 시각
    update_at         TEXT NOT NULL DEFAULT (datetime('now')),      -- 수정 시각
    start_at          TEXT,                                         -- 작업 시작 시각
    done_at           TEXT,                                         -- 완료 시각
    UNIQUE (project_sn, num)
);

-- 완료 조건. 태스크를 Done 처리하는 기준
CREATE TABLE tbl_task_criterion (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 완료 조건 번호
    task_sn          INTEGER NOT NULL REFERENCES tbl_task(sn) ON DELETE CASCADE,      -- 태스크
    content          TEXT NOT NULL,                                 -- 조건 내용
    is_done          INTEGER NOT NULL DEFAULT 0,                    -- 충족 여부
    sort             INTEGER NOT NULL DEFAULT 0,                    -- 표시 순서
    create_at        TEXT NOT NULL DEFAULT (datetime('now'))        -- 생성 시각
);

-- 태스크 의존 관계. task가 depend_task의 완료를 기다린다
CREATE TABLE tbl_map_task_dependency (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 의존 관계 번호
    task_sn          INTEGER NOT NULL REFERENCES tbl_task(sn) ON DELETE CASCADE,      -- 기다리는 태스크 (예: #133)
    depend_task_sn   INTEGER NOT NULL REFERENCES tbl_task(sn) ON DELETE CASCADE,      -- 먼저 끝나야 하는 태스크 (예: #128)
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    UNIQUE (task_sn, depend_task_sn)
);


-- =====================================================================
-- 7. 실행 (Run · Session · 컨텍스트 · 리뷰)
-- =====================================================================

-- Run. 태스크의 실행 시도 1회
CREATE TABLE tbl_run (
    sn                 INTEGER PRIMARY KEY AUTOINCREMENT,           -- Run 번호 (내부)
    project_sn         INTEGER NOT NULL REFERENCES tbl_project(sn) ON DELETE CASCADE, -- 프로젝트
    task_sn            INTEGER NOT NULL REFERENCES tbl_task(sn) ON DELETE CASCADE,    -- 태스크
    member_sn          INTEGER NOT NULL REFERENCES tbl_member(sn) ON DELETE RESTRICT,  -- 실행한 멤버
    num                INTEGER NOT NULL,                            -- 화면 표시 번호 (Run #81)
    status             TEXT NOT NULL DEFAULT 'queued' CHECK (status IN ('queued','starting','running','waiting','review','completed','failed','cancelled')),  -- 상태 (#9): queued(대기열) | starting(세션 시작 중) | running(실행 중) | waiting(판단 · 승인 대기로 멈춤) | review(결과 검토 중) | completed(성공) | failed(실패) | cancelled(사용자 · Orch가 중지)
    start_by           TEXT NOT NULL DEFAULT 'orch' CHECK (start_by IN ('orch','user','retry','lead')),  -- 시작한 쪽: orch(Orch 배정) | user(사용자 시작) | retry(실패 후 자동 재시도) | lead(리드가 하위 작업으로 요청 · #67)
    retry_run_sn       INTEGER REFERENCES tbl_run(sn) ON DELETE SET NULL,              -- 재시도 대상인 이전 Run
    parent_run_sn      INTEGER REFERENCES tbl_run(sn) ON DELETE CASCADE,               -- 상위(리드) Run · 하위 작업일 때 (#67)
    spawn_mode         TEXT CHECK (spawn_mode IN ('sub','fork','runner')),  -- 하위 작업 방식: NULL(일반 Run) | sub(실행기 내장 서브에이전트) | fork(부모 컨텍스트 상속) | runner(OrchStack 임시 하위 Run)
    tier               TEXT CHECK (tier IN ('S','M','L')),  -- 모델 등급 (runner): S(소형) | M(중형) | L(대형) · 규칙 엔진이 kind로 판정
    brief              TEXT,                                        -- 받은 @TASK 원문 (하위 작업)
    kind               TEXT CHECK (kind IN ('explore','search','format','test','implement','fix','design','review','debug')),  -- 하위 작업 종류 (@TASK kind) · 규칙 엔진이 이 값으로 tier를 정한다 (#67)
    child_seq          INTEGER,                                     -- 리드 Run 안의 하위 순번 (1부터) · 화면 표시 T{task.num}.{child_seq}
    paths              TEXT,                                        -- 하위 작업 허용 경로 JSON 배열 [{"path","source","at"}] · source = brief(처음 받음) | ask(@ASK로 추가) | violation(종료 후 발견한 범위 밖 변경) · 규칙 엔진 겹침 검사 (#67)
    wait_run_sn        INTEGER REFERENCES tbl_run(sn) ON DELETE SET NULL,               -- paths가 겹쳐 끝나기를 기다리는 하위 Run (queued · waits) · 끝나면 NULL
    wait_glob          TEXT,                                        -- 겹친 경로 (예: src/api/*) · 화면 연결선 라벨
    runtime_sn         INTEGER REFERENCES tbl_runtime(sn) ON DELETE SET NULL,          -- 사용한 실행기
    connection_sn      INTEGER REFERENCES tbl_connection(sn) ON DELETE SET NULL,       -- 사용한 연결
    model_code         TEXT,                                        -- 사용한 모델 ID (기록용 사본)
    effort             TEXT,                                        -- 추론 강도
    is_fallback        INTEGER NOT NULL DEFAULT 0,                  -- 폴백 연결로 실행했는지
    result_summary     TEXT,                                        -- 결과 요약 (예: tests 12/12)
    fail_code          TEXT,                                        -- 실패 분류 (예: lint, test, timeout, paths_violation(허용 경로 밖 변경 · #67), retry_exhausted)
    fail_detail        TEXT,                                        -- 실패 상세
    branch             TEXT,                                        -- 작업 브랜치
    workdir            TEXT,                                        -- 이 Run이 만든 worktree 경로 (repo 모드면 NULL)
    workdir_clean_at   TEXT,                                        -- worktree · 임시 브랜치 정리 시각 (workdir 있음 + NULL + 종료된 Run = 정리 대상)
    file_json          TEXT,                                        -- 변경 파일 JSON 배열 [{path, change_kind, additions, deletions}] · change_kind = A(추가) | M(수정) | D(삭제) | R(이름 변경)
    review_member_sn   INTEGER REFERENCES tbl_member(sn) ON DELETE SET NULL,  -- 반려한 멤버 (fail_code = rejected일 때 · 반려 횟수 = 그 태스크의 rejected Run 수)
    review_reason      TEXT,                                        -- 반려 사유 (예: 접근성 라벨 누락 2곳)
    context_limit      INTEGER,                                     -- 컨텍스트 한도 (예: 128000)
    start_at           TEXT,                                        -- 시작 시각
    end_at             TEXT,                                        -- 종료 시각
    create_at          TEXT NOT NULL DEFAULT (datetime('now')),     -- 생성 시각
    UNIQUE (project_sn, num)
);

-- 세션. Run 안에서 CLI가 연 실행 세션
CREATE TABLE tbl_session (
    sn                  INTEGER PRIMARY KEY AUTOINCREMENT,          -- 세션 번호 (내부)
    run_sn              INTEGER NOT NULL REFERENCES tbl_run(sn) ON DELETE CASCADE,    -- Run
    member_sn           INTEGER NOT NULL REFERENCES tbl_member(sn) ON DELETE RESTRICT, -- 멤버 (run.member_sn 과 같음 · 멤버별 세션 번호 UNIQUE 에 필요해 둔다)
    num                 INTEGER NOT NULL,                           -- 멤버별 표시 번호 (Session #12)
    provider_session_id TEXT,                                       -- CLI가 준 세션 ID (이어가기에 사용)
    status              TEXT NOT NULL DEFAULT 'starting' CHECK (status IN ('starting','active','stopped','failed')),  -- 상태 (#9): starting(시작 중) | active(사용 중) | stopped(정상 종료 · 교체) | failed(오류로 종료)
    is_resumed          INTEGER NOT NULL DEFAULT 0,                 -- 이전 세션을 이어서 시작했는지
    bootstrap_token     INTEGER NOT NULL DEFAULT 0,                 -- 세션 시작에 든 토큰
    rotate_reason       TEXT,                                       -- 교체로 멈춘 이유 (예: 컨텍스트 80% 초과 · 같은 Run 안에서 새 세션)
    start_at            TEXT NOT NULL DEFAULT (datetime('now')),    -- 시작 시각
    end_at              TEXT                                        -- 종료 시각
);


-- 컨텍스트 목록. 모델 호출 1회에 보낸 컨텍스트 묶음 (ContextManifest) · 출처 목록은 source_json
CREATE TABLE tbl_context_manifest (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 목록 번호
    run_sn           INTEGER NOT NULL REFERENCES tbl_run(sn) ON DELETE CASCADE,       -- Run
    session_sn       INTEGER REFERENCES tbl_session(sn) ON DELETE SET NULL,            -- 세션
    budget_token     INTEGER NOT NULL,                              -- 허용 예산(토큰)
    source_json      TEXT,                                          -- 이 호출에 보낸 출처 JSON 배열 (순서 = 조립 순서) [{kind, ref_label, ref_sn, ref_version, content_hash, token_count, is_repeat, sort}] · kind = preset(Instruction preset · 고정 버전) | instruction(개별 지침 · tbl_profile_file) | repo_rule(저장소 AGENTS.md · CLAUDE.md) | task(태스크 · 완료 조건) | file(코드 파일) · ref_label = 표시 이름 (예: AGENT.md, src/lib/auth.ts) · ref_sn = 원본 번호 (repo_rule · file은 NULL) · content_hash = 보낸 내용의 sha256 앞 16자 · is_repeat = 같은 Run의 이전 호출에 같은 (kind, ref_label, content_hash)가 있었다 · token_count = 이 호출에 보낸 추정 토큰
    create_at        TEXT NOT NULL DEFAULT (datetime('now'))        -- 생성 시각
);


-- 계약. 여러 태스크가 함께 쓰는 약속 (API · 스키마 · 공유 컴포넌트 · 설정 · 의존성 · 라우트 · 환경 변수)
--   연쇄 수정은 계약 버전으로 추적하고, 소비 태스크에는 마지막으로 본 버전 이후의 차이만 보낸다
CREATE TABLE tbl_contract (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 계약 번호
    project_sn       INTEGER NOT NULL REFERENCES tbl_project(sn) ON DELETE CASCADE,  -- 프로젝트
    kind             TEXT NOT NULL CHECK (kind IN ('api','schema','export','config','dep','route','env')),  -- 종류: api(HTTP · RPC) | schema(DB) | export(공유 모듈 · 컴포넌트) | config(설정) | dep(패키지 의존성) | route(화면 경로) | env(환경 변수)
    key              TEXT NOT NULL,                                 -- 식별 키 (예: api:POST /auth/refresh, export:$lib/ui/Input)
    version          INTEGER NOT NULL DEFAULT 1,                    -- 버전 (변경될 때마다 +1)
    summary          TEXT NOT NULL,                                 -- 현재 계약 한 줄 요약 (영어 · 최대 200자 · 예: resp {access, refresh, expires_in:sec})
    delta_json       TEXT,                                          -- 버전별 한 줄 차이 JSON 배열 [{version, change_code, delta, run_sn, create_at}] · change_code = add | modify | remove | breaking(호환 깨짐 · 소비 태스크 재작업 필요) · delta = 영어 · 최대 200자 (예: expires_in ms -> sec) · 소비 태스크에는 seen_version 이후만 보낸다
    owner_task_sn    INTEGER REFERENCES tbl_task(sn) ON DELETE SET NULL,  -- 계약을 만든 · 책임지는 태스크
    update_run_sn    INTEGER REFERENCES tbl_run(sn) ON DELETE SET NULL,   -- 마지막으로 바꾼 Run
    update_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 수정 시각
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    UNIQUE (project_sn, key)
);

-- 보고 항목. @REPORT 블록을 항목 단위로 쪼개 저장 (받는 쪽은 필요한 기계 항목만 받고, 사용자 보고서는 사람 칸 + 시스템 사실로 조립)
--   변경 파일 · 줄 수 · 테스트 · 토큰 · 시간은 시스템이 수집하므로 여기에 두지 않는다 (tbl_run.file_json · tbl_log_token)
CREATE TABLE tbl_report_item (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 항목 번호
    run_sn           INTEGER NOT NULL REFERENCES tbl_run(sn) ON DELETE CASCADE,  -- 보고한 Run
    task_sn          INTEGER NOT NULL REFERENCES tbl_task(sn) ON DELETE CASCADE, -- 태스크
    kind             TEXT NOT NULL CHECK (kind IN ('ac','change','impact','verify','risk','result','area','review','unverified','left')),  -- 종류 · 기계 칸(라우팅): ac(완료 조건 체크) | change(계약 변경 · tbl_contract 참조) | impact(다른 태스크 영향) | verify(검증 방법) | risk(위험) · 사람 칸(보고서 전용 · Agent에 전달 안 함): result(결과 1~2문장) | area(주요 변경 · ref_key=영역) | review(검토할 지점 · ref_key=대상) | unverified(미확인 · 필수) | left(남은 문제)
    ref_key          TEXT,                                          -- 참조 키: ac=조건 순번(1,2…, 원문 보존용 · 실제 연결은 criterion_sn) · change=계약 키(api:POST /auth/refresh) · impact=대상 태스크 번호(T130)
    criterion_sn     INTEGER REFERENCES tbl_task_criterion(sn) ON DELETE SET NULL,  -- ac 대상 완료 조건 (순번은 조건을 지우면 어긋나서 sn으로 연결)
    code             TEXT NOT NULL,                                 -- 결과 코드: ac=ok | fail | skip · change=add | modify | remove · impact=retest | rework | review | ctx_update · left/risk=짧은 분류 코드(todo | blocked | perf | security …)
    value            TEXT,                                          -- 값: 기계 칸 = 영어 · 최대 120자 · 자유 서술 금지 / 사람 칸 = 화면 언어 · 보고서 문체(대화체 금지) · 최대 300자
    target_task_sn   INTEGER REFERENCES tbl_task(sn) ON DELETE SET NULL,  -- impact 대상 태스크 (규칙 엔진이 연결)
    contract_sn      INTEGER REFERENCES tbl_contract(sn) ON DELETE SET NULL,  -- change 대상 계약
    is_routed        INTEGER NOT NULL DEFAULT 0,                    -- 받는 쪽에 전달 완료 여부
    create_at        TEXT NOT NULL DEFAULT (datetime('now'))        -- 생성 시각
);


-- 태스크 ↔ 계약. 태스크가 쓰는(소비) 또는 만드는(제공) 계약 · 마지막으로 본 버전
CREATE TABLE tbl_map_task_contract (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 연결 관계 번호
    task_sn          INTEGER NOT NULL REFERENCES tbl_task(sn) ON DELETE CASCADE,  -- 태스크
    contract_sn      INTEGER NOT NULL REFERENCES tbl_contract(sn) ON DELETE CASCADE,  -- 계약
    role             TEXT NOT NULL CHECK (role IN ('provide','consume')),  -- 관계: provide(제공 · 바꿀 수 있음) | consume(사용 · 바뀌면 영향)
    seen_version     INTEGER NOT NULL DEFAULT 0,                    -- 이 태스크가 마지막으로 받은 버전 (이후 delta만 전달)
    UNIQUE (task_sn, contract_sn)
);


-- 요청 연결. 다이어그램의 연결선 · 컨텍스트 메뉴 '연결하기'로도 생성
--   의존 관계 · 배정은 tbl_map_task_dependency · tbl_task.member_sn 에 저장하고,
--   여기에는 멤버 사이에 오가는 요청(검증 · 리뷰 · 완료 보고 · 위임)만 둔다
CREATE TABLE tbl_interaction (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 요청 번호
    project_sn       INTEGER NOT NULL REFERENCES tbl_project(sn) ON DELETE CASCADE, -- 프로젝트
    task_sn          INTEGER REFERENCES tbl_task(sn) ON DELETE CASCADE,               -- 관련 태스크
    issue_sn         INTEGER REFERENCES tbl_issue(sn) ON DELETE SET NULL,              -- 관련 이슈 (위임일 때)
    from_member_sn   INTEGER REFERENCES tbl_member(sn) ON DELETE SET NULL,             -- 요청한 멤버 (사용자가 만들면 NULL)
    to_member_sn     INTEGER REFERENCES tbl_member(sn) ON DELETE SET NULL,             -- 요청받은 멤버
    kind             TEXT NOT NULL CHECK (kind IN ('request_verification','request_review','completion_report','delegate')),  -- 종류: request_verification(검증 요청 · 예: 진 → 하린) | request_review(리뷰 요청 · → Reviewer) | completion_report(완료 보고 · → Orch) | delegate(위임 · Orch → 이슈)
    status           TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open','live','done','cancelled')),  -- 상태: open(대기 · 상대가 아직 시작 안 함) | live(처리 중 · 연결선 강조 표시) | done(완료) | cancelled(취소)
    create_by        TEXT NOT NULL DEFAULT 'member' CHECK (create_by IN ('member','orch','user')),  -- 만든 쪽: member(멤버) | orch(Orch) | user(사용자 · 연결하기 메뉴)
    user_sn              INTEGER REFERENCES tbl_user(sn) ON DELETE SET NULL,              -- 만든 사용자
    note             TEXT,                                          -- 요청 내용 (예: Task #130 QA 대기 해제 요청)
    event_sn         INTEGER REFERENCES tbl_log_event(sn) ON DELETE SET NULL,  -- 이 행을 만든 이벤트 (projection 재구축 · 중복 방지)
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    close_at         TEXT                                           -- 끝난 시각
);


-- 다이어그램. 배치 방식 · 확대 비율 · 표시 범위 · 노드 위치 · 사용자별 화면 설정(조회 전용, domain 아님)
CREATE TABLE tbl_diagram (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 보기 설정 번호
    project_sn       INTEGER NOT NULL REFERENCES tbl_project(sn) ON DELETE CASCADE, -- 프로젝트
    user_sn              INTEGER NOT NULL REFERENCES tbl_user(sn) ON DELETE CASCADE,     -- 사용자
    layout_mode      TEXT NOT NULL DEFAULT 'auto' CHECK (layout_mode IN ('auto','grid','manual')),  -- 배치: auto(자동 배치) | grid(격자 정렬) | manual(사용자가 옮긴 위치 유지)
    zoom_percent     INTEGER NOT NULL DEFAULT 100,                  -- 확대 비율(%)
    is_show_capability INTEGER NOT NULL DEFAULT 1,                  -- 스킬 · MCP · 도구 노드 표시
    is_show_done     INTEGER NOT NULL DEFAULT 0,                    -- 완료된 태스크 표시
    node_json        TEXT,                                          -- 노드 위치 JSON 배열 [{node_type, node_sn, pos_x, pos_y, is_collapsed}] · 사용자가 옮긴 위치만 (없으면 자동 배치) · node_type = project | issue | task | member | skill | mcp | tools (node_sn이 가리키는 테이블 · 대상이 여러 테이블이라 FK 없음 → 태스크 · 멤버 삭제 시 앱에서 함께 정리)
    update_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 수정 시각
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    UNIQUE (project_sn, user_sn)
);


-- =====================================================================
-- 8. Orch 요청(판단 · 승인 · 제안) · 대화
-- =====================================================================


-- 요청. Orch가 사람에게 묻거나 알리는 카드 하나 · PM Dock · 결정 패널 · 알림의 '확인 필요'
--   kind별 option_json 구조 (응답에서는 파싱해 객체로 준다)
--     decision = 질문 배열 [{title, body, code_snippet, ref, options:[{code, label, note, is_recommended, is_selected}], answer_text, is_delegate, answer_at}]
--     approval = {rule_title, detail}
--     proposal = 다른 선택지 배열 [{label, kind, member_sn?, task_sn?}]
CREATE TABLE tbl_ask (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 요청 번호
    project_sn       INTEGER NOT NULL REFERENCES tbl_project(sn) ON DELETE CASCADE,   -- 프로젝트
    issue_sn         INTEGER REFERENCES tbl_issue(sn) ON DELETE SET NULL,              -- 관련 이슈
    task_sn          INTEGER REFERENCES tbl_task(sn) ON DELETE SET NULL,               -- 관련 태스크
    run_sn           INTEGER REFERENCES tbl_run(sn) ON DELETE SET NULL,                -- 관련 Run (판단 · 승인이면 답을 기다리는 Run)
    member_sn        INTEGER REFERENCES tbl_member(sn) ON DELETE SET NULL,             -- 묻거나 요청한 멤버 · 제안의 대상 멤버 (Orch가 만든 제안은 대상 멤버)
    kind             TEXT NOT NULL CHECK (kind IN ('decision','approval','proposal')),  -- 종류: decision(판단 요청 · 에이전트가 사람에게 묻는 질문 묶음) | approval(승인 요청 · 승인 규칙에 걸린 동작) | proposal(Orch 제안 · 배정 · 재시도 · 가드 정지 등 규칙 엔진이 만든 카드)
    action           TEXT CHECK (action IN ('pr_create','pr_merge','dependency_add','run_extend','external_message','assign','retry','close_issue','next_issue','fallback','guard_stop')),  -- 동작: approval = pr_create(PR 생성) | pr_merge(PR 병합) | dependency_add(새 의존성 추가) | run_extend(Run 제한 시간 연장) | external_message(팀 외부로 메시지) · proposal = assign(다음 태스크 배정) | retry(실패 후 재시도) | close_issue(이슈 완료 처리) | next_issue(다음 이슈 제안) | fallback(연결 전환 · 폴백) | guard_stop(루프 가드 정지) · decision = NULL
    level            INTEGER NOT NULL DEFAULT 1,                    -- 작업 레벨 0 ~ 4 (판단 요청은 L2 이상)
    title            TEXT NOT NULL,                                 -- 요약 제목 (예: 재전송 제한 · #130 QA를 하린에게 배정)
    reason           TEXT,                                          -- 근거 (제안 · 가드 정지의 이유 · Orch가 대신 결정한 근거)
    option_json      TEXT,                                          -- 질문 · 선택지 · 답 / 승인 내용 / 다른 선택지 (위 구조)
    status           TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','writing','answered','orch_decided','cancelled','approved','denied','expired','proposed','auto_done','user_done','changed','stopped','dismissed')),  -- 상태 (kind마다 쓰는 값이 다르다): decision = pending(답변 대기 · 타이머 진행) | writing(사용자가 작성 중 · 타이머 멈춤) | answered(사용자가 답함) | orch_decided(시간 초과로 Orch가 대신 결정) | cancelled(취소) · approval = pending(승인 대기) | approved(승인) | denied(거부) | expired(시간 초과로 만료) · proposal = proposed(제안 · 타이머 진행 중) | auto_done(타이머 후 자동 진행) | user_done(사용자가 바로 진행) | changed(다른 선택으로 변경) | stopped(사용자가 멈춤 · 가드 정지) | dismissed(무시 · 나중에)
    is_timer_pause   INTEGER NOT NULL DEFAULT 0,                    -- 타이머 멈춤 (작성 중)
    is_review_needed INTEGER NOT NULL DEFAULT 0,                    -- 사용자가 재검토하기로 표시 (판단 요청)
    deadline_at      TEXT,                                          -- 이 시각이 지나면 판단 = Orch가 결정 · 승인 = 만료 · 제안 = 자동 진행
    decide_by        TEXT CHECK (decide_by IN ('user','orch')),     -- 결정한 쪽: user(사용자) | orch(사용자 대신 Orch)
    user_sn          INTEGER REFERENCES tbl_user(sn) ON DELETE SET NULL,              -- 처리한 사용자 (자동 진행이면 NULL)
    streak_count     INTEGER NOT NULL DEFAULT 0,                    -- 연속 자동 진행 횟수 (제안 · 예: 3 / 10)
    guard_code       TEXT,                                          -- 걸린 루프 가드 코드 (가드 정지 제안 · auto_streak · reject_loop …)
    event_sn         INTEGER REFERENCES tbl_log_event(sn) ON DELETE SET NULL,  -- 이 요청을 만든 이벤트
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    decide_at        TEXT                                           -- 처리 시각
);


-- 메시지. 프로젝트 Orch 대화 안의 말 · 작업 제안 · 명령 결과 (member_sn = 보낸 멤버 · Orch가 보낸 것이면 그 Orch)
CREATE TABLE tbl_message (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 메시지 번호
    project_sn       INTEGER NOT NULL REFERENCES tbl_project(sn) ON DELETE CASCADE,   -- 프로젝트 (Orch 대화는 프로젝트마다 하나)
    sender_type      TEXT NOT NULL CHECK (sender_type IN ('user','orch','member')),  -- 보낸 쪽: user(사용자) | orch(Orch) | member(멤버)
    member_sn        INTEGER REFERENCES tbl_member(sn) ON DELETE SET NULL,             -- 보낸 멤버 (user면 NULL)
    kind             TEXT NOT NULL DEFAULT 'text' CHECK (kind IN ('text','work_proposal','command_result','runtime_instruction')),  -- 종류: text(일반 메시지) | work_proposal(작업 제안 카드) | command_result(명령 실행 결과) | runtime_instruction(실행 중 멤버에게 보낸 지시)
    content          TEXT,                                          -- 본문
    payload_json     TEXT,                                          -- 카드 내용 JSON (제안한 이슈 · 태스크 · 의존 관계 등)
    proposal_status  TEXT CHECK (proposal_status IN ('draft','proceeded','cancelled')),  -- 작업 제안 상태: draft(초안) | proceeded(진행 · 이슈 · 태스크 생성됨) | cancelled(취소)
    task_sn          INTEGER REFERENCES tbl_task(sn) ON DELETE SET NULL,               -- 언급한 태스크 (@Task)
    run_sn           INTEGER REFERENCES tbl_run(sn) ON DELETE SET NULL,                -- 언급하거나 전달한 Run (@Run)
    create_at        TEXT NOT NULL DEFAULT (datetime('now'))        -- 보낸 시각
);

-- 첨부 파일. 메시지 · 질문 답변 · 태스크에 붙인 파일
CREATE TABLE tbl_attachment (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 첨부 번호
    owner_type       TEXT NOT NULL CHECK (owner_type IN ('message','ask','task')),  -- 붙인 곳: message(메시지) | ask(요청 · 판단 답변) | task(태스크)
    owner_sn         INTEGER NOT NULL,                              -- 붙인 곳의 sn
    user_sn              INTEGER REFERENCES tbl_user(sn) ON DELETE SET NULL,              -- 올린 사용자
    file_name        TEXT NOT NULL,                                 -- 파일 이름
    mime_type        TEXT,                                          -- 파일 형식 (예: image/png)
    file_size        INTEGER NOT NULL DEFAULT 0,                    -- 크기(바이트)
    file_path        TEXT NOT NULL,                                 -- 저장 경로
    create_at        TEXT NOT NULL DEFAULT (datetime('now'))        -- 올린 시각
);


-- =====================================================================
-- 9. 알림
-- =====================================================================


-- 알림 목록. 상단 벨에 쌓이는 알림
CREATE TABLE tbl_notification (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 알림 번호
    workspace_sn     INTEGER NOT NULL REFERENCES tbl_workspace(sn) ON DELETE CASCADE, -- 워크스페이스
    user_sn              INTEGER NOT NULL REFERENCES tbl_user(sn) ON DELETE CASCADE,     -- 받는 사용자
    event_code       TEXT NOT NULL CHECK (event_code IN ('decision_request','approval_request','orch_decided','run_failed','guard_stop','context_warn','quota_low','budget_80','budget_over','connection_error','fallback_used','pr','task_done','daily_summary')),  -- 이벤트 (tbl_workspace.notify_json 의 event_code 와 같음)
    actor_type       TEXT NOT NULL DEFAULT 'system' CHECK (actor_type IN ('orch','member','system')),  -- 알린 쪽: orch(Orch) | member(멤버) | system(시스템)
    member_sn        INTEGER REFERENCES tbl_member(sn) ON DELETE SET NULL,             -- 알린 멤버
    title            TEXT NOT NULL,                                 -- 제목
    body             TEXT,                                          -- 설명
    ref_type         TEXT CHECK (ref_type IN ('ask','run','task','connection')),  -- 바로가기 대상: ask(판단 · 승인 · 제안) | run(Run) | task(태스크) | connection(연결)
    ref_sn           INTEGER,                                       -- 바로가기 대상의 sn (여러 테이블이라 FK 없음 · 대상이 지워지면 화면에 '삭제된 항목' 표시)
    is_action        INTEGER NOT NULL DEFAULT 0,                    -- 확인 필요(답하기 · 승인 버튼) 알림인지
    is_read          INTEGER NOT NULL DEFAULT 0,                    -- 읽음 여부
    event_sn         INTEGER REFERENCES tbl_log_event(sn) ON DELETE SET NULL,  -- 이 알림을 만든 이벤트 (같은 이벤트로 두 번 만들지 않는다)
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 생성 시각
    read_at          TEXT                                           -- 읽은 시각
);


-- =====================================================================
-- 10. 기록 (쌓이기만 하는 테이블)
-- =====================================================================

-- 이벤트 저장소 (#10 Command → Event → Projection · 초안, #10에서 확정). 상태 변경의 원본 · 쌓이기만 한다
--   화면용 테이블(tbl_task.status, tbl_log_activity, tbl_interaction …)은 이 기록에서 만든 projection
--   @TASK · @REPORT · @ASK 블록 원문은 payload_json 에 저장 (모델에 다시 보내지 않음)
CREATE TABLE tbl_log_event (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 이벤트 번호 (전역 순서 · 재연결 커서 · 쓰기는 워크스페이스당 1개 트랜잭션씩 직렬화해야 번호 순서 = 커밋 순서가 된다)
    workspace_sn              INTEGER NOT NULL REFERENCES tbl_workspace(sn) ON DELETE CASCADE,  -- 워크스페이스
    project_sn       INTEGER REFERENCES tbl_project(sn) ON DELETE CASCADE,  -- 프로젝트 (워크스페이스 전체 이벤트면 NULL)
    aggregate_type   TEXT NOT NULL CHECK (aggregate_type IN ('workspace','project','issue','task','run','session','profile','template','member','team','ask','contract','connection','preset','message','skill')),  -- 대상 종류: workspace | project | issue | task | run | session | profile(에이전트 프로필 · 폴백 체인) | template | member | team | ask(판단 · 승인 · 제안) | contract | connection | preset | message(Orch 대화 메시지) | skill
    aggregate_sn     INTEGER NOT NULL,                              -- 대상 번호 (aggregate_type 테이블의 sn · FK 없음 · 대상이 지워져도 기록은 남는다)
    seq              INTEGER NOT NULL,                              -- 대상별 순번 (1부터 · 동시 수정 충돌 감지)
    event_type       TEXT NOT NULL,                                 -- 이벤트 이름 (PascalCase · 예: TaskCreated, AgentAssigned, RunStarted, ReportReceived, ContractChanged, DecisionAnswered)
    event_version    INTEGER NOT NULL DEFAULT 1,                    -- payload 구조 버전
    payload_json     TEXT NOT NULL,                                 -- 이벤트 내용 JSON (블록 원문 포함 · 영어 키)
    command_id       TEXT,                                          -- 이 이벤트를 만든 명령 ID (같은 명령 재전송 시 중복 방지)
    command_idx      INTEGER NOT NULL DEFAULT 0,                    -- 한 명령이 만든 이벤트 중 순서 (0부터 · 예: MoveTask → TaskMoved 0, AgentAssigned 1)
    actor_type       TEXT NOT NULL CHECK (actor_type IN ('user','orch','member','system')),  -- 발생시킨 쪽: user(사용자) | orch(Orch) | member(멤버) | system(시스템 · 규칙 엔진)
    user_sn              INTEGER REFERENCES tbl_user(sn) ON DELETE SET NULL,     -- 발생시킨 사용자
    member_sn        INTEGER REFERENCES tbl_member(sn) ON DELETE SET NULL,    -- 발생시킨 멤버
    run_sn           INTEGER REFERENCES tbl_run(sn) ON DELETE SET NULL,       -- 관련 Run
    create_at        TEXT NOT NULL DEFAULT (datetime('now')),       -- 발생 시각
    UNIQUE (aggregate_type, aggregate_sn, seq)
);

-- Run 로그. 실시간 로그 · 도구 호출 · 테스트 · Git (하단 Ops 패널)
CREATE TABLE tbl_log_run (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 로그 번호
    run_sn           INTEGER NOT NULL REFERENCES tbl_run(sn) ON DELETE CASCADE,       -- Run
    session_sn       INTEGER REFERENCES tbl_session(sn) ON DELETE SET NULL,            -- 세션
    kind             TEXT NOT NULL DEFAULT 'log' CHECK (kind IN ('log','tool_call','test','git','command')),  -- 종류: log(일반 로그) | tool_call(도구 호출) | test(테스트 실행) | git(Git 작업) | command(Orch 명령)
    level            TEXT NOT NULL DEFAULT 'info' CHECK (level IN ('info','warn','error')),  -- 수준: info(정보) | warn(경고) | error(오류)
    title            TEXT,                                          -- 짧은 이름 (예: edit_file, pnpm test auth)
    message          TEXT,                                          -- 내용
    create_at        TEXT NOT NULL DEFAULT (datetime('now'))        -- 기록 시각
);

-- 토큰 기록. 모델 호출 1회마다 (TokenLedger)
CREATE TABLE tbl_log_token (
    sn                 INTEGER PRIMARY KEY AUTOINCREMENT,           -- 기록 번호
    run_sn             INTEGER NOT NULL REFERENCES tbl_run(sn) ON DELETE CASCADE,     -- Run
    session_sn         INTEGER REFERENCES tbl_session(sn) ON DELETE SET NULL,          -- 세션
    connection_sn      INTEGER REFERENCES tbl_connection(sn) ON DELETE SET NULL,       -- 사용한 연결
    manifest_sn        INTEGER REFERENCES tbl_context_manifest(sn) ON DELETE SET NULL, -- 이 호출에 보낸 컨텍스트 목록
    model_code         TEXT,                                        -- 모델 ID
    token_input        INTEGER NOT NULL DEFAULT 0,                  -- 새 입력 토큰
    token_cache_read   INTEGER NOT NULL DEFAULT 0,                  -- 캐시 읽기 토큰
    token_cache_write  INTEGER NOT NULL DEFAULT 0,                  -- 캐시 쓰기 토큰
    token_output       INTEGER NOT NULL DEFAULT 0,                  -- 출력 토큰
    context_token      INTEGER,                                     -- 호출 시점의 컨텍스트 크기(추정) · NULL = 모름
    cost_usd_micro     INTEGER NOT NULL DEFAULT 0,                  -- 비용 (1달러 = 1,000,000) · 구독은 0
    usage_source       TEXT NOT NULL DEFAULT 'provider' CHECK (usage_source IN ('provider','estimated')),  -- 토큰 값 출처: provider(제공자가 보고한 실측) | estimated(OrchStack 추정 · 제공자가 안 줄 때) (#16)
    create_at          TEXT NOT NULL DEFAULT (datetime('now'))      -- 호출 시각
);

-- 활동 기록. Activity 타임라인 (배정 · 지시 · 리뷰 · 메시지 · 결정 · 시스템) · tbl_log_event에서 만든 화면용 기록(projection)
CREATE TABLE tbl_log_activity (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 기록 번호
    project_sn       INTEGER REFERENCES tbl_project(sn) ON DELETE SET NULL,            -- 프로젝트
    team_sn          INTEGER REFERENCES tbl_team(sn) ON DELETE SET NULL,               -- 팀
    task_sn          INTEGER REFERENCES tbl_task(sn) ON DELETE SET NULL,               -- 관련 태스크
    run_sn           INTEGER REFERENCES tbl_run(sn) ON DELETE SET NULL,                -- 관련 Run
    actor_type       TEXT NOT NULL CHECK (actor_type IN ('user','orch','member','system')),  -- 한 쪽: user(사용자) | orch(Orch) | member(멤버) | system(시스템)
    user_sn              INTEGER REFERENCES tbl_user(sn) ON DELETE SET NULL,              -- 한 사용자
    member_sn        INTEGER REFERENCES tbl_member(sn) ON DELETE SET NULL,             -- 한 멤버
    target_member_sn INTEGER REFERENCES tbl_member(sn) ON DELETE SET NULL,             -- 받은 멤버 (예: Orch → 진)
    kind             TEXT NOT NULL CHECK (kind IN ('TASK_INSTRUCTION','ASSIGN','RUN','TOOL_CALL','MESSAGE','TEST','REQUEST_VERIFICATION','REVIEW','DECISION','SYSTEM')),  -- 종류: TASK_INSTRUCTION(태스크 지시) | ASSIGN(배정) | RUN(Run 시작 · 종료) | TOOL_CALL(도구 호출) | MESSAGE(메시지) | TEST(테스트) | REQUEST_VERIFICATION(검증 요청) | REVIEW(리뷰 · 반려) | DECISION(결정) | SYSTEM(시스템 · 한도 경고 등)
    title            TEXT NOT NULL,                                 -- 한 줄 요약
    body             TEXT,                                          -- 상세
    ref_type         TEXT CHECK (ref_type IN ('ask','review','message','interaction')),  -- 펼쳐 볼 대상: ask | review | message | interaction (ref_sn이 가리키는 테이블)
    ref_sn           INTEGER,                                       -- 펼쳐 볼 대상의 sn (여러 테이블이라 FK 없음 · 대상이 지워지면 화면에 '삭제된 항목' 표시)
    event_sn         INTEGER REFERENCES tbl_log_event(sn) ON DELETE SET NULL,  -- 원본 이벤트 (이 행을 만든 tbl_log_event)
    create_at        TEXT NOT NULL DEFAULT (datetime('now'))        -- 기록 시각
);

-- 감사 기록. 키 · 정책 · 연결 · 스킬 변경과 차단 내역
CREATE TABLE tbl_log_audit (
    sn               INTEGER PRIMARY KEY AUTOINCREMENT,             -- 기록 번호
    workspace_sn     INTEGER NOT NULL REFERENCES tbl_workspace(sn) ON DELETE CASCADE, -- 워크스페이스
    actor_type       TEXT NOT NULL CHECK (actor_type IN ('user','orch','member','system')),  -- 한 쪽: user(사용자) | orch(Orch) | member(멤버) | system(시스템)
    user_sn              INTEGER REFERENCES tbl_user(sn) ON DELETE SET NULL,              -- 한 사용자
    member_sn        INTEGER REFERENCES tbl_member(sn) ON DELETE SET NULL,             -- 한 멤버 (차단된 멤버 포함)
    run_sn           INTEGER REFERENCES tbl_run(sn) ON DELETE SET NULL,                -- 관련 Run
    kind             TEXT NOT NULL CHECK (kind IN ('KEY','POLICY','CONNECTION','INSTALL','UPDATE','BLOCK')),  -- 종류: KEY(키 추가 · 삭제) | POLICY(권한 · 정책 변경) | CONNECTION(연결 변경) | INSTALL(스킬 · MCP 설치) | UPDATE(업데이트) | BLOCK(차단된 동작)
    title            TEXT NOT NULL,                                 -- 내용 (예: git reset --hard origin/main 차단)
    detail           TEXT,                                          -- 사유 · 상세 (예: Destructive git)
    create_at        TEXT NOT NULL DEFAULT (datetime('now'))        -- 기록 시각
);


-- =====================================================================
-- 11. 인덱스 (목록 · 필터에 자주 쓰는 조회)
-- =====================================================================

CREATE INDEX idx_connection_workspace   ON tbl_connection (workspace_sn, kind);
CREATE INDEX idx_member_team            ON tbl_member (team_sn);
CREATE INDEX idx_issue_project          ON tbl_issue (project_sn, status);
CREATE INDEX idx_task_project_status    ON tbl_task (project_sn, status);
CREATE INDEX idx_task_member            ON tbl_task (member_sn, status);
CREATE INDEX idx_task_issue             ON tbl_task (issue_sn);
CREATE INDEX idx_run_task               ON tbl_run (task_sn);
CREATE INDEX idx_run_member             ON tbl_run (member_sn, create_at);
CREATE INDEX idx_run_parent             ON tbl_run (parent_run_sn);
CREATE INDEX idx_run_wait               ON tbl_run (wait_run_sn);
CREATE INDEX idx_session_run            ON tbl_session (run_sn);
CREATE INDEX idx_ask_status             ON tbl_ask (project_sn, kind, status);
CREATE INDEX idx_message_project        ON tbl_message (project_sn, create_at);
CREATE INDEX idx_attachment_owner       ON tbl_attachment (owner_type, owner_sn);
CREATE INDEX idx_notification_user      ON tbl_notification (user_sn, is_read, create_at);
CREATE INDEX idx_notification_ref       ON tbl_notification (ref_type, ref_sn);                  -- 대상 처리 시(판단 요청 답변 등) 확인 필요 알림 해제
CREATE INDEX idx_log_run_run            ON tbl_log_run (run_sn, create_at);
CREATE INDEX idx_log_token_run          ON tbl_log_token (run_sn);
CREATE INDEX idx_log_token_create       ON tbl_log_token (create_at);                  -- 보관 기간(retention_token_day) 정리
CREATE INDEX idx_log_run_create         ON tbl_log_run (create_at);                    -- 보관 기간(retention_run_log_day) 정리
CREATE INDEX idx_log_activity_task      ON tbl_log_activity (task_sn, create_at);
CREATE INDEX idx_log_activity_member    ON tbl_log_activity (member_sn, create_at);
CREATE INDEX idx_interaction_task       ON tbl_interaction (task_sn, status);
CREATE INDEX idx_interaction_project    ON tbl_interaction (project_sn, status);
CREATE INDEX idx_report_item_run       ON tbl_report_item (run_sn, kind);
CREATE INDEX idx_report_item_target    ON tbl_report_item (target_task_sn, is_routed);
CREATE INDEX idx_map_task_contract     ON tbl_map_task_contract (contract_sn, role);
CREATE INDEX idx_map_profile_preset    ON tbl_map_profile_preset (preset_sn);
CREATE INDEX idx_preset_latest         ON tbl_instruction_preset (kind, preset_key, is_latest);
CREATE INDEX idx_log_audit_workspace    ON tbl_log_audit (workspace_sn, create_at);

-- 외래 키 조회 · CASCADE 삭제용 (SQLite는 FK 인덱스를 자동으로 만들지 않음)
CREATE INDEX idx_task_dependency_depend ON tbl_map_task_dependency (depend_task_sn);
CREATE INDEX idx_context_manifest_run   ON tbl_context_manifest (run_sn);
CREATE INDEX idx_task_criterion_task    ON tbl_task_criterion (task_sn, sort);
CREATE INDEX idx_issue_parent           ON tbl_issue (parent_sn);
CREATE INDEX idx_map_profile_skill      ON tbl_map_profile_skill (skill_sn);
CREATE INDEX idx_map_profile_mcp        ON tbl_map_profile_mcp (mcp_sn);
CREATE INDEX idx_template_revision      ON tbl_template_revision (template_sn);


-- =====================================================================
-- 12. 유일 제약 (NULL이 끼면 UNIQUE가 막지 못해서 COALESCE 식 인덱스로 건다)
-- =====================================================================

CREATE UNIQUE INDEX ux_session_num      ON tbl_session (member_sn, num);                     -- 멤버별 세션 번호
CREATE UNIQUE INDEX ux_run_child        ON tbl_run (parent_run_sn, child_seq);        -- 리드 Run 안 하위 순번 (NULL 여러 개 허용)
CREATE UNIQUE INDEX ux_event_command     ON tbl_log_event (command_id, command_idx);             -- NULL은 여러 개 허용 · 같은 명령 재전송은 첫 이벤트에서 충돌
CREATE INDEX idx_log_event_project      ON tbl_log_event (project_sn, sn);

-- 13. 외래 키 인덱스 (SQLite는 FK에 인덱스를 자동으로 만들지 않는다 · 삭제 규칙 · 조인 성능용)
CREATE INDEX idx_fk_agent_profile_connection_sn ON tbl_agent_profile (connection_sn);
CREATE INDEX idx_fk_agent_profile_model_sn ON tbl_agent_profile (model_sn);
CREATE INDEX idx_fk_agent_profile_runtime_sn ON tbl_agent_profile (runtime_sn);
CREATE INDEX idx_fk_agent_profile_workspace_sn ON tbl_agent_profile (workspace_sn);
CREATE INDEX idx_fk_ask_member_sn ON tbl_ask (member_sn);
CREATE INDEX idx_fk_ask_run_sn ON tbl_ask (run_sn);
CREATE INDEX idx_fk_ask_task_sn ON tbl_ask (task_sn);
CREATE INDEX idx_fk_ask_issue_sn ON tbl_ask (issue_sn);
CREATE INDEX idx_fk_ask_user_sn ON tbl_ask (user_sn);
CREATE INDEX idx_fk_attachment_user_sn ON tbl_attachment (user_sn);
CREATE INDEX idx_fk_connection_runtime_sn ON tbl_connection (runtime_sn);
CREATE INDEX idx_fk_connection_user_sn ON tbl_connection (user_sn);
CREATE INDEX idx_fk_context_manifest_session_sn ON tbl_context_manifest (session_sn);
CREATE INDEX idx_fk_contract_owner_task_sn ON tbl_contract (owner_task_sn);
CREATE INDEX idx_fk_contract_update_run_sn ON tbl_contract (update_run_sn);
CREATE INDEX idx_fk_diagram_user_sn ON tbl_diagram (user_sn);
CREATE INDEX idx_fk_instruction_preset_copy_from_sn ON tbl_instruction_preset (copy_from_sn);
CREATE INDEX idx_fk_instruction_preset_project_sn ON tbl_instruction_preset (project_sn);
CREATE INDEX idx_fk_instruction_preset_user_sn ON tbl_instruction_preset (user_sn);
CREATE INDEX idx_fk_interaction_from_member_sn ON tbl_interaction (from_member_sn);
CREATE INDEX idx_fk_interaction_issue_sn ON tbl_interaction (issue_sn);
CREATE INDEX idx_fk_interaction_to_member_sn ON tbl_interaction (to_member_sn);
CREATE INDEX idx_fk_interaction_user_sn ON tbl_interaction (user_sn);
CREATE INDEX idx_fk_issue_user_sn ON tbl_issue (user_sn);
CREATE INDEX idx_fk_log_activity_event_sn ON tbl_log_activity (event_sn);
CREATE INDEX idx_fk_log_activity_project_sn ON tbl_log_activity (project_sn);
CREATE INDEX idx_fk_log_activity_run_sn ON tbl_log_activity (run_sn);
CREATE INDEX idx_fk_log_activity_target_member_sn ON tbl_log_activity (target_member_sn);
CREATE INDEX idx_fk_log_activity_team_sn ON tbl_log_activity (team_sn);
CREATE INDEX idx_fk_log_activity_user_sn ON tbl_log_activity (user_sn);
CREATE INDEX idx_fk_log_audit_member_sn ON tbl_log_audit (member_sn);
CREATE INDEX idx_fk_log_audit_run_sn ON tbl_log_audit (run_sn);
CREATE INDEX idx_fk_log_audit_user_sn ON tbl_log_audit (user_sn);
CREATE INDEX idx_fk_log_event_member_sn ON tbl_log_event (member_sn);
CREATE INDEX idx_fk_log_event_run_sn ON tbl_log_event (run_sn);
CREATE INDEX idx_fk_log_event_user_sn ON tbl_log_event (user_sn);
CREATE INDEX idx_fk_log_event_workspace_sn ON tbl_log_event (workspace_sn);
CREATE INDEX idx_fk_log_run_session_sn ON tbl_log_run (session_sn);
CREATE INDEX idx_fk_log_token_connection_sn ON tbl_log_token (connection_sn);
CREATE INDEX idx_fk_log_token_manifest_sn ON tbl_log_token (manifest_sn);
CREATE INDEX idx_fk_log_token_session_sn ON tbl_log_token (session_sn);
CREATE INDEX idx_fk_member_profile_sn ON tbl_member (profile_sn);
CREATE INDEX idx_fk_member_template_sn ON tbl_member (template_sn);
CREATE INDEX idx_fk_message_member_sn ON tbl_message (member_sn);
CREATE INDEX idx_fk_message_run_sn ON tbl_message (run_sn);
CREATE INDEX idx_fk_message_task_sn ON tbl_message (task_sn);
CREATE INDEX idx_fk_notification_member_sn ON tbl_notification (member_sn);
CREATE INDEX idx_fk_notification_workspace_sn ON tbl_notification (workspace_sn);
CREATE INDEX idx_fk_project_team_sn ON tbl_project (team_sn);
CREATE INDEX idx_fk_project_workspace_sn ON tbl_project (workspace_sn);
CREATE INDEX idx_fk_report_item_task_sn ON tbl_report_item (task_sn);
CREATE INDEX idx_fk_report_item_contract_sn ON tbl_report_item (contract_sn);
CREATE INDEX idx_fk_run_connection_sn ON tbl_run (connection_sn);
CREATE INDEX idx_fk_run_retry_run_sn ON tbl_run (retry_run_sn);
CREATE INDEX idx_fk_run_runtime_sn ON tbl_run (runtime_sn);
CREATE INDEX idx_fk_interaction_event_sn ON tbl_interaction (event_sn);
CREATE INDEX idx_fk_notification_event_sn ON tbl_notification (event_sn);
CREATE INDEX idx_fk_report_item_criterion_sn ON tbl_report_item (criterion_sn);
CREATE INDEX idx_fk_skill_source_sn ON tbl_skill (source_sn);
CREATE INDEX idx_fk_skill_source_workspace_sn ON tbl_skill_source (workspace_sn);
CREATE INDEX idx_fk_task_user_sn ON tbl_task (user_sn);
CREATE INDEX idx_fk_team_workspace_sn ON tbl_team (workspace_sn);
CREATE INDEX idx_fk_template_user_sn ON tbl_template (user_sn);
CREATE INDEX idx_fk_template_workspace_sn ON tbl_template (workspace_sn);
CREATE INDEX idx_fk_template_revision_profile_sn ON tbl_template_revision (profile_sn);
CREATE INDEX idx_fk_template_revision_user_sn ON tbl_template_revision (user_sn);
CREATE INDEX idx_fk_workspace_user_sn ON tbl_workspace (user_sn);
