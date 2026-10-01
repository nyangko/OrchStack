# UI 컴포넌트 지침

SvelteKit · Svelte 5 · shadcn-svelte(Bits UI) · Tailwind CSS 프로젝트의 화면과 컴포넌트에 적용한다.
`.svelte` 를 새로 만들거나 고치기 전에 이 문서를 기준으로 삼는다.

> 이 프로젝트에서 서비스 이름은 `orch` 다. 아래의 `cms/<종류>/` 는 `src/lib/components/orch/<종류>/` 로 읽는다.

---

## 0. 원칙 한 줄

**화면은 페이지에서 위에서 아래로 한 번 읽고 고칠 수 있어야 한다.**
파일 수를 줄이고 한 화면의 가독성을 지키는 것이 컴포넌트를 잘게 나누는 것보다 우선한다.

---

## 1. 층과 책임

| 층 | 위치 | 책임 |
|---|---|---|
| Page | `src/routes/**` | 화면 구성과 배치. 데이터를 사용자 동작에 잇는다 |
| Primitive | `src/lib/components/ui/` | 범용 부품. 업무 데이터를 모른다 |
| Layout | `src/lib/components/layout/` | 화면 틀(헤더 · 사이드바) |
| Feature | `src/lib/components/cms/<종류>/` | 이 서비스 전용 부품. 완결된 사용자 작업 하나를 맡는다 |
| Logic | `src/lib/logic/` | 계산 · 필터링 · 변환 · 상태 판정. TypeScript 순수 함수 |
| State | `src/lib/state/` | 여러 화면이 나눠 쓰는 상태(`*.svelte.ts`) |
| API | `src/lib/api/` | 서버 호출 |
| Config | `src/lib/config/` | 라벨 · 상태 이름 · 고정 목록 |

- `cms` 는 이 프로젝트의 이름이다. 프로젝트마다 서비스 이름으로 바꿔도 된다(예: `erp/`, `app/`). 규칙은 같다.
- 층을 새로 만들지 않는다. `sections/` · `widgets/` · `containers/` · `blocks/` 같은 정리용 폴더를 두지 않는다.

```
src/lib/components/
├─ ui/                 범용 부품 (shadcn 조각 + 한 줄 판)
├─ layout/             AppHeader · Sidebar …
└─ cms/
   ├─ dialog/          XxxDialog.svelte
   ├─ menu/            XxxActions.svelte
   ├─ viewer/          XxxViewer.svelte
   ├─ form/            (필요해질 때)
   └─ table/           (필요해질 때)
```

---

## 2. `ui/` — 범용 부품

### 2-1. 무엇이 들어가나

- shadcn-svelte 로 설치한 부품과 그 조각 파일(`dialog-content.svelte` …).
- shadcn 에 없지만 **업무 데이터를 모르는** 범용 부품. 예: combobox(쳐서 찾는 한 값 고르기), pick-menu(목록을 좁히는 거르개 · 정렬 단추), page-title, icon, confirm.
- 업무 이름이 붙은 것은 `ui/` 에 두지 않는다. `ProjectCombobox` 는 `ui/` 가 아니다. 옵션은 부르는 쪽이 만든다.

### 2-2. 「한 줄 판」(`*-single.svelte`)

자주 쓰는 shadcn 부품에는 조각을 다 짜 둔 **한 줄 판**을 하나 둔다.

- 반복되는 틀(머리 · 제목 · 설명 · 아이콘 · 단추 · 오류)은 **props** 로 받는다.
- 내용(몸)만 **`children`** 으로 받는다.
- 부르는 쪽은 조각(`Root` · `Content` · `Header` …)을 다시 짜지 않는다.

```svelte
<!-- 부르는 쪽 -->
<Dialog bind:open title="자격 추가" description="…" icon={Award}
        actions={[{ label: '취소', variant: 'ghost', onclick: close },
                  { label: '추가', icon: Check, type: 'submit', form: 'license-form', busy }]}>
  <form id="license-form">…</form>
</Dialog>

<Field label="이름" for="name" required error={errors.name}>
  <Input id="name" bind:value={name} />
</Field>
```

- 한 줄 판을 두는 부품의 예: dialog · card · field · select · table · tabs · sheet · toggle-group · empty · tooltip.
- 한 줄 판으로 안 되는 특수한 경우에만 조각을 직접 조합한다.

### 2-3. `index.ts` 내보내기

조각과 한 줄 판을 함께 내보낸다. **한 줄 판은 부품 이름 그대로** 내보낸다.

```ts
import Root from './dialog.svelte';
import Content from './dialog-content.svelte';
// …
import Single from './dialog-single.svelte';

export {
  Root, Content, /* … 조각 */
  /** 한 줄로 쓰는 창 — `<Dialog bind:open title=… actions={…}>몸</Dialog>` */
  Single as Dialog,
  Content as DialogContent, /* … */
};
export type { DialogAction } from './dialog-single.svelte';
```

부르는 쪽은 `import { Dialog } from '$lib/components/ui/dialog'` 하나만 쓴다.

### 2-3-1. 부르는 쪽은 이름 붙은 태그만 쓴다 (2026-10-01 사용자 권장)

- 페이지 · `orch/` 에서는 `<Kanban.Root>` · `<Dialog.Header>` 같은 점 표기를 쓰지 않는다. `<KanbanBoard>` · `<KanbanHeader>` · `<DialogHeader>` 처럼 **태그 이름만으로** 읽히게 한다.
- 그래서 `import * as X from …` 를 쓰지 않고, 쓰는 조각을 이름으로 가져온다: `import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent } from '$lib/components/ui/dropdown-menu'`.
- `index.ts` 는 모든 조각을 `<부품><조각>` 이름으로 내보낸다(`DropdownMenuItem` · `ContextMenuEntries`).
- 부품 이름 그대로(`Dialog`)는 한 줄 판이 있으면 한 줄 판, 없으면 Root 다. 한 줄 판이 생기면 Root 는 `<부품>Root` 로 내보낸다.

### 2-4. 생김새와 색

- 생김새 차이는 `tailwind-variants` 의 `tv()` variant 로 낸다. 클래스 문자열을 부르는 쪽에서 반복해 넘기지 않는다.
- 색 · 간격 · 둥글기는 **토큰에서만** 가져온다. `layout.css` 의 CSS 변수와 shadcn semantic 토큰(`bg-card` · `text-muted-foreground` · `border-border`)을 쓴다.
- 상태 색(통과 · 보류 · 실패 · 대기 · 안내)은 `tone-*` 같은 유틸리티 하나로 정의하고, 부품은 그 이름만 쓴다.
- 새 hex, 임의 px(`w-[137px]`)는 예외다. 써야 하면 이유를 주석으로 남긴다. 그 전에 다음 순서로 찾는다: 기존 부품 → 토큰 → 유틸리티 → 그래도 없으면 임의값.

### 2-5. 새 `ui/` 부품을 들일 때

- shadcn 에 있으면 shadcn 으로 설치한다. 직접 만들지 않는다.
- 기존 shadcn 부품을 **이름만 바꾸려고 감싸지 않는다.** 공통 API · 동작 · 디자인 규칙을 실제로 세울 때만 wrapper 를 만든다.
- 같은 Bits UI 조합이 두 곳 이상에서 반복되면 그때 `ui/` 로 올린다. 「나중에 쓸 것 같다」는 근거가 아니다.

---

## 3. `layout/` — 화면 틀

- 앱 헤더 · 사이드바처럼 여러 화면을 감싸는 틀만 둔다.
- 한 구역(예: 관리자 화면 묶음)에서만 쓰는 틀은 컴포넌트가 아니라 그 구역의 `+layout.svelte` 에 바로 적는다.

---

## 4. `cms/<종류>/` — 서비스 전용 부품

### 4-1. 폴더는 업무가 아니라 「부품 종류」로 나눈다

- 맞음: `cms/dialog/ProjectDialog.svelte`, `cms/menu/ProjectActions.svelte`, `cms/viewer/PdfViewer.svelte`
- 틀림: `cms/project/ProjectDialog.svelte`, `cms/common/…`, `cms/upload/…`
- 파일 이름은 `<대상><종류>` 로 짓는다: `XxxDialog` · `XxxActions` · `XxxViewer` · `XxxForm` · `XxxTable`.

### 4-2. 꺼내는 기준

다음 중 **하나 이상**일 때만 `cms/` 로 꺼낸다.

1. 실제로 **두 곳 이상**에서 쓰인다.
2. 스스로 상태와 동작을 가진 **완결된 사용자 작업**이다. 예: 업로드 · 일괄 가져오기 · 주소 고르기 · 등록 창.
3. 차트 · 에디터 · 뷰어 · 복합 입력처럼 **독립 UI 로서 충분히 복잡**하다.

다음은 꺼내는 이유가 아니다.

- 코드가 길다
- 데이터 계산이 복잡하다(→ `logic/` 으로)
- 화면에서 카드 · 섹션 · 머리처럼 덩어리로 보인다
- 나중에 다시 쓸 것 같다

꺼내기 전에 확인한다.

- 책임을 한 문장으로 말할 수 있다.
- 부르는 자리에서 이름과 props 만 보고 쓰임을 알 수 있다.
- 꺼낸 뒤 페이지가 실제로 더 읽기 쉬워진다.

### 4-3. 창(Dialog) 컴포넌트는 창 전체를 갖는다

- `open` 을 `$bindable` 로 받는다.
- `ui/` 의 Dialog 한 줄 판으로 틀 · 머리 · 몸 · 발을 **전부** 그린다.
- 페이지는 한 줄만 적는다: `<ProjectDialog bind:open />`
- 「틀(Dialog.Root · Content)은 페이지에, 안쪽만 컴포넌트에」로 나누지 않는다.
- 열 때마다 새로 시작해야 하는 값(입력 초기화 등)은 컴포넌트 안에서 처리한다. `$effect.pre` 로 `open` 을 감시하고, 이 코드는 스크립트 맨 끝에 둔다.

```svelte
<!-- 페이지 -->
<script lang="ts">
  import ProjectDialog from '$lib/components/cms/dialog/ProjectDialog.svelte';
  let creating = $state(false);
</script>

<Button onclick={() => (creating = true)}>새 프로젝트</Button>
<ProjectDialog bind:open={creating} />
```

### 4-4. 업무 동작은 컴포넌트 안에서 끝낸다

- 서버 호출(`lib/api`), 알림(toast), 확인 창, 화면 이동은 그 컴포넌트가 직접 한다.
- 부르는 쪽은 **데이터 하나**만 넘긴다. 예: `<ProjectActions {project} />`
- 두 번 눌러 요청이 두 번 가지 않게, 요청 중 상태(`busy`)를 컴포넌트가 가진다.

### 4-5. 정리

- 옮기거나 바꾼 뒤에는 **사용처가 0인 컴포넌트를 찾아 지운다.**
- shadcn 부품 안의 쓰지 않는 조각(예: `Dialog.Trigger`)을 지울지는 따로 정한다.

---

## 5. 페이지 작성

### 5-1. 한 번 쓰는 것은 그 자리에 펼친다

화면의 카드 · 섹션 · 머리를 이유로 새 컴포넌트를 만들지 않는다.

```svelte
<!-- 이렇게 한다 -->
<Card title="보유 자격" meta={licenses.length}>…</Card>
<Card title="서류" meta={docs.length}>…</Card>

<!-- 이렇게 하지 않는다 -->
<LicenseSectionCard />
<DocumentListHeader />
<ActionRequiredCard />
```

필드가 10개면 10개를 그 자리에 적는다.

### 5-2. 판단 질문

새 `.svelte` 를 만들기 전에 묻는다.

> **이 파일을 없애고 부모 페이지에 넣었을 때 실제로 이해하기 어려워지는가?**

아니면 만들지 않는다.

### 5-3. 스니펫과 작은 함수

- `{#snippet}` 은 두 경우에만 쓴다.
  1. 라이브러리가 요구하는 `child` · `children`
  2. **한 파일 안에서** 실제로 여러 번 되풀이되는 틀
- 스니펫을 함수나 다른 스니펫의 인자로 넘기지 않는다. 사람이 따라 읽을 수 없게 된다.
- 반복은 스니펫이 아니라 **데이터 + `{#each}`** 로 돌린다.
- 한 번만 쓰는 작은 함수를 만들지 않는다. 계산 · 변환이 필요하면 `lib/logic/` 에 순수 함수로 둔다.

### 5-4. 화면 순서대로 읽힌다

- 마크업 순서가 사용자가 보는 순서와 같아야 한다.
- 열어 봐야만 화면을 알 수 있는 불투명한 컴포넌트 목록으로 페이지를 만들지 않는다.

---

## 6. 컴포넌트 API

- props 는 **지금 부르는 곳이 실제로 쓰는 것만** 받는다.
- 받는 것은 props 와 `children` 이다. 스니펫 여러 개를 받아 조립하는 구조를 만들지 않는다.
- `mode` · `kind` · `type` · `variant` 와 설정 객체의 조합으로 동작이 갈리는 범용 부품을 만들지 않는다. 한 줄 판에 켜고 끄는 값(`interactive` · `selected` …)이 쌓이면 두 부품으로 나눌지 먼저 따진다.
- 관련된 입력 부품은 이름과 동작을 맞춘다: `value` · `size` · `disabled` · `label` · `description` · `error`.
- 비슷한 일을 하는 부품끼리는 생김새 · 목록 규칙을 맞춘다. 예: 폼 안의 Select 와 Combobox.
- 부르는 자리에서 사용자 작업이 드러나는 API 를 고른다.
- 클래스 문자열을 숨기려고 컴포넌트를 만들지 않는다. 토큰 · 유틸리티 · variant 중 맞는 것을 쓴다.

### 비슷한 부품을 가르는 기준

| 부품 | 쓰는 곳 |
|---|---|
| `Select` | 폼에서 짧은 목록 중 하나를 고르는 입력 |
| `Combobox` | 폼에서 긴 목록을 쳐서 찾는 입력 |
| `PickMenu` | 목록을 좁히는 단추(거르개 · 정렬 · 쪽당 개수) |
| `Tabs` | 같은 대상의 다른 면을 바꿔 보기 |
| `ToggleGroup` | 목록 거르개 · 보기 방식 전환 |
| `Dialog` | 하던 일을 멈추고 끝내야 하는 작업 |
| `Sheet` | 목록을 보면서 옆에서 하는 작업 |

---

## 7. 코드 쓰는 법

### 7-1. 파일 머리 주석

- 첫 줄은 **굵게, 한 줄로** 「무엇을 하는 부품인지」 적는다.
- 비슷한 부품과 어떻게 다른지, 왜 이렇게 만들었는지를 이어서 적는다.
- props 마다 한 줄 주석을 단다. 값이 비었을 때 어떻게 되는지도 적는다.

```svelte
<script lang="ts">
  /**
   * **한 값 고르기.** 거르개 · 정렬 · 쪽당 개수가 전부 같은 일이라 같은 컴포넌트다.
   * 폼의 입력 칸은 이것이 아니라 `Select` · `Combobox` 다 — 저쪽은 「적어 넣는 값」, 이쪽은 「목록을 좁히는 단추」.
   */
  let {
    label,
    /** 「전체」로 되돌릴 수 있나 — 거르개는 풀 수 있어야 하고, 쪽당 개수는 아니다 */
    clearable = true,
  }: { label: string; clearable?: boolean } = $props();
</script>
```

### 7-2. 타입

- 밖에서 쓰는 타입(`DialogAction` · `ComboboxOption`)은 `<script lang="ts" module>` 에서 export 하고, `index.ts` 에서 다시 내보낸다.
- `any` 를 쓰지 않는다.

### 7-3. Svelte 5

- `$props()` · `$state()` · `$derived()` · `$bindable()` 를 쓴다.
- `$effect` 안에서 같은 상태를 읽고 쓰지 않는다. 무한 루프가 난다. 필요하면 `untrack` 으로 끊는다.
- `$state` 배열은 프록시다. push 한 원본 객체로 `indexOf` 하면 찾지 못한다.

### 7-4. 따옴표 · 서식

- 프로젝트 전체에서 하나로 정한다. shadcn 이 설치하는 파일도 같은 규칙으로 맞춘다.
- 포매터(Prettier)에 맡기고 손으로 맞추지 않는다.

---

## 8. 입력 · 비동기

- 사용자가 적은 값은 오류가 나도 지우지 않는다.
- 요청 중에는 단추를 막고 도는 표시를 보인다. 두 번 요청하지 않는다.
- 오류는 그 칸 아래에 보인다. 오류가 있으면 설명 대신 오류가 같은 자리에 선다.
- 지우기는 먼저 상태 바꾸기(종료 · 해지)를 제안한다. 정말 지울 때는 확인을 받고, 「되돌리기」를 준다.

---

## 9. 접근성 · 반응형

- 입력에는 라벨을 잇는다(`for` / `id`). 아이콘만 있는 단추에는 `aria-label` 을 단다.
- 꾸밈 아이콘에는 `aria-hidden="true"` 를 단다.
- 키보드로 열고 닫고 고를 수 있어야 한다. Bits UI 부품의 기본 동작을 깨지 않는다.
- 좁은 화면에서 넘치거나 겹치지 않는지 브라우저에서 본다. `svelte-check` 와 `build` 가 통과해도 화면은 깨질 수 있다.

---

## 10. 새 파일을 만들기 전 점검

1. `ui/` 에 그 자리가 이미 있는가? 있으면 조합한다.
2. `cms/` 에 같은 일을 하는 부품이 있는가? 있으면 쓴다.
3. 꺼내는 기준(4-2) 중 하나를 만족하는가? 아니면 페이지에 펼친다.
4. 계산 · 변환이면 컴포넌트가 아니라 `logic/` 인가?
5. 폴더는 종류(dialog · menu · viewer · form · table) 기준인가?
6. 이름 · props 만 보고 부르는 자리에서 쓰임이 드러나는가?

---

## 11. 완료 보고에 적을 것

- 바꾼 화면 · 파일
- **새로 만든 컴포넌트와 꺼낸 이유**(4-2 의 어느 조건인지)
- 지운 컴포넌트(사용처 0)
- 쓴 임의값과 그 이유
- 브라우저에서 확인한 것
- 요청 범위 밖이라 손대지 않은 것
