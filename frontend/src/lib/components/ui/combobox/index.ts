import Root from "./combobox.svelte";
import Trigger from "./combobox-trigger.svelte";
import Content from "./combobox-content.svelte";
import Item from "./combobox-item.svelte";
import { Input as Search, List, Empty, Group, Separator } from "$lib/components/ui/command/index.js";

export {
	Root,
	Trigger,
	Content,
	Search,
	List,
	Empty,
	Group,
	Item,
	Separator,
	//
	Root as Combobox,
	Trigger as ComboboxTrigger,
	Content as ComboboxContent,
	Search as ComboboxSearch,
	List as ComboboxList,
	Empty as ComboboxEmpty,
	Group as ComboboxGroup,
	Item as ComboboxItem,
	Separator as ComboboxSeparator,
};
