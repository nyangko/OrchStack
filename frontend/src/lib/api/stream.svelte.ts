/// 프로젝트 이벤트 클라이언트 1개 (#46 · #91): snapshot → SSE → 끊기면 재연결(?after=) → 다시 열릴 때 events?after=로 빠진 것 보충.
/// 적용은 스토어($lib/project.svelte.ts)가 하고, 여기서는 순서 · 중복(sn) · 연결 상태만 책임진다.
/// 끊김 감지: 서버가 15초마다 ping 이벤트를 보낸다. timeout 동안 아무것도 안 오면 끊긴 것으로 보고 직접 다시 연다
/// (프록시가 백엔드 종료를 브라우저에 전하지 않는 경우가 있어 EventSource의 자체 재연결만 믿지 않는다).
import { api } from './client';
import { apiUrl } from './env';
import type { Schema } from './types';

export type ApiEvent = Schema<'EventOut'>;
export type ApiSnapshot = Schema<'Snapshot'>;
export type StreamState = 'loading' | 'live' | 'reconnecting' | 'error';

/// EventSource는 이름 있는 이벤트(event:)를 리스너가 있는 이름만 전달한다. 서버가 보내는 종류 전부 — 여기 없는 종류는 브라우저가 버린다 (= 모르는 이벤트 무시).
export const EVENT_TYPES = [
	'TaskCreated', 'TaskUpdated', 'TaskMoved', 'TaskDeleted', 'AgentAssigned', 'AgentUnassigned',
	'IssueCreated', 'IssueUpdated', 'IssueDeleted',
	'MemberCreated', 'MemberUpdated', 'MemberDeleted', 'TeamCreated', 'TeamUpdated', 'TeamDeleted',
	'RunStarted', 'RunCancelled', 'ReviewRequested', 'RunApproved', 'RunRejected', 'RunMoved', 'SessionMoved',
	'ProfileCreated', 'ProfileUpdated', 'ProfileDeleted'
] as const;

/** 서버 ping 간격 15초 × 2 + 여유. */
const DEFAULT_TIMEOUT = 40_000;
/** 연결 자체가 거부됐을 때(백엔드 내려감) 다시 시도하는 간격. */
const RETRY = 3_000;

export class ProjectStream {
	state = $state<StreamState>('loading');
	/** 마지막으로 적용한 이벤트 sn. 이보다 작거나 같은 것은 중복이라 버린다. */
	lastSn = $state(0);
	#es?: EventSource;
	#closed = false;
	#catching = false;
	#watchdog?: ReturnType<typeof setTimeout>;
	#retry?: ReturnType<typeof setTimeout>;

	constructor(
		readonly sn: number,
		private onSnapshot: (s: ApiSnapshot) => void,
		private onEvent: (e: ApiEvent) => void | Promise<void>,
		private timeout = DEFAULT_TIMEOUT
	) {}

	/// snapshot을 받고 스트림을 연다. snapshot 실패면 error.
	async start() {
		const { data } = await api.GET('/projects/{sn}/snapshot', { params: { path: { sn: this.sn } } });
		if (!data || this.#closed) {
			this.state = 'error';
			return;
		}
		this.lastSn = data.last_event_sn;
		this.onSnapshot(data);
		this.#open();
	}

	#open() {
		if (this.#closed) return;
		this.#es?.close();
		// ?after= : 직접 다시 연 연결에는 브라우저가 Last-Event-ID를 안 붙이므로 우리가 아는 마지막 sn을 넘긴다
		const es = new EventSource(`${apiUrl}/projects/${this.sn}/stream?after=${this.lastSn}`);
		this.#es = es;
		es.onopen = () => {
			// 재연결이면 끊긴 사이를 보충한다 (서버도 after로 보내지만, 밀려 버린(lagged) 것은 DB에서만 받을 수 있다)
			if (this.state === 'reconnecting') void this.catchUp();
			this.state = 'live';
			this.#bump();
		};
		// 연결이 끊기면 브라우저가 자동으로 다시 붙는다. 아예 거부되면(CLOSED · 백엔드 내려감) RETRY 뒤에 우리가 다시 연다
		es.onerror = () => {
			if (this.#closed) return;
			this.state = 'reconnecting';
			if (es.readyState === EventSource.CLOSED) {
				clearTimeout(this.#retry);
				this.#retry = setTimeout(() => this.#open(), RETRY);
			}
		};
		es.addEventListener('ping', () => this.#bump());
		for (const t of EVENT_TYPES) {
			es.addEventListener(t, (m) => {
				this.#bump();
				void this.#take(JSON.parse((m as MessageEvent).data) as ApiEvent);
			});
		}
		this.#bump();
	}

	/// 감시 타이머 재설정. timeout 안에 아무것도 안 오면 끊긴 것으로 보고 다시 연다
	#bump() {
		clearTimeout(this.#watchdog);
		this.#watchdog = setTimeout(() => {
			if (this.#closed) return;
			this.state = 'reconnecting';
			this.#open();
		}, this.timeout);
	}

	/// 순서 · 중복 검사 후 적용. 보충 중이면 보충이 끝난 뒤 순서대로 들어온다
	async #take(ev: ApiEvent) {
		if (ev.sn <= this.lastSn) return;
		this.lastSn = ev.sn;
		await this.onEvent(ev);
	}

	/// lastSn 다음부터 전부 받아 적용한다. 500건이면 마지막 sn으로 이어서 부른다
	async catchUp() {
		if (this.#catching) return;
		this.#catching = true;
		try {
			for (;;) {
				const { data } = await api.GET('/projects/{sn}/events', { params: { path: { sn: this.sn }, query: { after: this.lastSn } } });
				if (!data) return;
				for (const ev of data) await this.#take(ev);
				if (data.length < 500) return;
			}
		} finally {
			this.#catching = false;
		}
	}

	close() {
		this.#closed = true;
		clearTimeout(this.#watchdog);
		clearTimeout(this.#retry);
		this.#es?.close();
	}
}
