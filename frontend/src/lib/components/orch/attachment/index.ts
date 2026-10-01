import AttachmentAction from "./AttachmentAction.svelte";
import AttachmentActions from "./AttachmentActions.svelte";
import AttachmentContent from "./AttachmentContent.svelte";
import AttachmentDescription from "./AttachmentDescription.svelte";
import AttachmentGroup from "./AttachmentGroup.svelte";
import AttachmentMedia from "./AttachmentMedia.svelte";
import AttachmentTitle from "./AttachmentTitle.svelte";
import AttachmentTrigger from "./AttachmentTrigger.svelte";
import Attachment, {
	attachmentVariants,
	type AttachmentOrientation,
	type AttachmentSize,
	type AttachmentState,
} from "./Attachment.svelte";

export {
	attachmentVariants,
	type AttachmentSize,
	type AttachmentOrientation,
	type AttachmentState,
	Attachment,
	AttachmentGroup,
	AttachmentMedia,
	AttachmentContent,
	AttachmentTitle,
	AttachmentDescription,
	AttachmentActions,
	AttachmentAction,
	AttachmentTrigger,
};
