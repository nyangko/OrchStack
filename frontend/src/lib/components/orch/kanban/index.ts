/// 범용 칸반 — 열 · 카드를 태그로 조립한다: <KanbanBoard><KanbanColumn><KanbanColumnHeader>…<KanbanCard>…
/// 업무(태스크 · 상태)는 모른다. 열 값 · 카드 id · 카드 안 내용은 부르는 쪽이 정한다.
import KanbanBoard from "./KanbanBoard.svelte";
import KanbanColumn from "./KanbanColumn.svelte";
import KanbanColumnHeader from "./KanbanColumnHeader.svelte";
import KanbanColumnTitle from "./KanbanColumnTitle.svelte";
import KanbanColumnCount from "./KanbanColumnCount.svelte";
import KanbanColumnActions from "./KanbanColumnActions.svelte";
import KanbanColumnContent from "./KanbanColumnContent.svelte";
import KanbanCard from "./KanbanCard.svelte";
import KanbanCardHeader from "./KanbanCardHeader.svelte";
import KanbanCardContent from "./KanbanCardContent.svelte";
import KanbanCardFooter from "./KanbanCardFooter.svelte";

export type { KanbanId, KanbanValue } from "./context.js";
export {
	KanbanBoard,
	KanbanColumn,
	KanbanColumnHeader,
	KanbanColumnTitle,
	KanbanColumnCount,
	KanbanColumnActions,
	KanbanColumnContent,
	KanbanCard,
	KanbanCardHeader,
	KanbanCardContent,
	KanbanCardFooter,
};
