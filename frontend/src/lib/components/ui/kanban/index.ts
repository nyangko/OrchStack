import Root from "./kanban.svelte";
import Column from "./kanban-column.svelte";
import ColumnHeader from "./kanban-column-header.svelte";
import ColumnTitle from "./kanban-column-title.svelte";
import ColumnCount from "./kanban-column-count.svelte";
import ColumnActions from "./kanban-column-actions.svelte";
import ColumnContent from "./kanban-column-content.svelte";
import Item from "./kanban-item.svelte";
import ItemHeader from "./kanban-item-header.svelte";
import ItemContent from "./kanban-item-content.svelte";
import ItemFooter from "./kanban-item-footer.svelte";

export type { KanbanId, KanbanValue } from "./context.js";
export {
	Root,
	Column,
	ColumnHeader,
	ColumnTitle,
	ColumnCount,
	ColumnActions,
	ColumnContent,
	Item,
	ItemHeader,
	ItemContent,
	ItemFooter,
	//
	Root as Kanban,
	Column as KanbanColumn,
	ColumnHeader as KanbanColumnHeader,
	ColumnTitle as KanbanColumnTitle,
	ColumnCount as KanbanColumnCount,
	ColumnActions as KanbanColumnActions,
	ColumnContent as KanbanColumnContent,
	Item as KanbanItem,
	ItemHeader as KanbanItemHeader,
	ItemContent as KanbanItemContent,
	ItemFooter as KanbanItemFooter,
};
