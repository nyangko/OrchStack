/// 보드 · 열 상태를 부품끼리 나눈다 (Root → Column → Item · ColumnCount · ColumnContent).
import { getContext, setContext } from "svelte";

export type KanbanId = string | number;
export type KanbanValue = Record<string, KanbanId[]>;

const BOARD = Symbol("kanban-board");
const COLUMN = Symbol("kanban-column");

export const setBoard = (get: () => KanbanValue) => setContext(BOARD, get);
export const getBoard = () => getContext<() => KanbanValue>(BOARD);
export const setColumn = (get: () => string) => setContext(COLUMN, get);
export const getColumn = () => getContext<() => string>(COLUMN);
