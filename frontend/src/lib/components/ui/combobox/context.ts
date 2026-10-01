/// Root가 연 창을 Item이 닫을 수 있게 한다.
import { getContext, setContext } from "svelte";

const KEY = Symbol("combobox");
export const setCombobox = (close: () => void) => setContext(KEY, close);
export const getCombobox = () => getContext<() => void>(KEY);
