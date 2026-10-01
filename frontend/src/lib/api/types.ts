/// OpenAPI 스키마 타입 별칭. 화면 타입(mock.ts)은 겹치는 필드를 여기서 Pick해 서버와 이름이 어긋나지 않게 한다.
import type { components } from './schema';

export type Schema<K extends keyof components['schemas']> = components['schemas'][K];

export type ApiProject = Schema<'Project'>;
export type ApiIssue = Schema<'Issue'>;
export type ApiTask = Schema<'Task'>;
export type ApiRun = Schema<'Run'>;
export type ApiSession = Schema<'Session'>;
export type ApiTeam = Schema<'Team'>;
export type ApiMember = Schema<'Member'>;
export type ApiProfile = Schema<'Profile'>;
export type ApiTemplate = Schema<'Template'>;
export type ApiError = Schema<'ErrorBody'>;
