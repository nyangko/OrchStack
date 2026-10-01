/// Root의 값을 Item이 읽고 바꾼다. single = 하나(라디오), multiple = 여러 개(체크).
import { getContext, setContext } from "svelte";

export type ChoiceValue = string | number | null;
type Ctx = { multiple: () => boolean; has: (v: ChoiceValue) => boolean; toggle: (v: ChoiceValue) => void; disabled: () => boolean };
const KEY = Symbol("choice-cards");
export const setChoice = (c: Ctx) => setContext(KEY, c);
export const getChoice = () => getContext<Ctx>(KEY);
