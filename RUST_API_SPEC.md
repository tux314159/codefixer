# Rust API specification
# LZQ (slopped >:()

## General

- Base path: `/api/v1`
- Opaque cursors
- Personalized responses: `Cache-Control: private, no-store`
- Unknown or unauthorized private problems: `404`

## Internal request headers

Every `/api/v1/*` request includes:

```text
Authorization: Bearer <svelte-service-token>
Accept: application/json
X-Request-Id: <request-id>
Cookie: <session-cookie-name>=<session-id>
```

Rules:

- `Authorization` identifies the SvelteKit service and is required.
- `Cookie` identifies the current user and is optional.
- If `Cookie` is absent or invalid, handle the request as anonymous.
- Return the same `X-Request-Id` in the response.
- GET endpoints have no request body.

## Authentication

```text
POST /auth/login?next=/problems
POST /auth/logout
```

Rust owns OAuth and the session cookie. The cookie must be `Secure`, `HttpOnly`, `SameSite=Lax`,
and scoped to `/`.

- `/auth/login` receives the frontend return path.
- `/auth/logout` receives the session cookie and invalidates it.

## Enums

```ts
type ProblemType = 'Batch' | 'Interactive' | 'Communication';

type ProblemStatus = 'unattempted' | 'attempted' | 'partial' | 'solved';

type ProblemSort =
  | 'solves-desc'
  | 'solves-asc'
  | 'difficulty-asc'
  | 'difficulty-desc'
  | 'score-desc'
  | 'score-asc'
  | 'newest'
  | 'id-asc'
  | 'id-desc';
```

The `attempted` filter includes `attempted` and `partial`. Every sort uses problem ID as the final
tie-breaker.

## Schemas

```ts
interface ViewerSummary {
  id: string;
  username: string;
  displayName: string;
  avatarUrl: string | null;
  capabilities: string[];
}

interface ProblemSummary {
  id: string;
  title: string;
  difficulty: number | null;
  type: ProblemType;
  source: string;
  authors: string[];
  solves: number;
  score: number | null;
  status: ProblemStatus;
  createdAt: string;
  revision: string;
}

interface ProblemStatusCounts {
  all: number;
  unattempted: number;
  attempted: number;
  solved: number;
}

interface TagFacet {
  name: string;
  count: number;
}

interface ProblemCollectionPage {
  queryKey: string;
  catalogVersion: string;
  progressVersion: string | null;
  items: ProblemSummary[];
  nextCursor: string | null;
  counts?: ProblemStatusCounts;
  tagFacets?: TagFacet[];
}

interface ProblemAttachment {
  id: string;
  name: string;
  contentType: string;
  sizeBytes: number;
  url: string;
  expiresAt: string | null;
}

interface ProblemSubtask {
  id: string;
  score: number;
}

interface SubmissionPolicy {
  allowed: boolean;
  blockedReason: 'login_required' | 'submission_limit' | 'cooldown' | 'problem_closed' | null;
  allowedLanguages: string[];
  maxSourceBytes: number;
  cooldownSeconds: number;
  remainingSubmissions: number | null;
}

interface EditorialPolicy {
  available: boolean;
  reason: 'not_published' | 'contest_active' | 'solve_required' | null;
  url: string | null;
}

interface ProblemDetail extends ProblemSummary {
  statementHtml: string;
  tags: string[];
  limits: {
    timeMs: number;
    memoryBytes: number;
  };
  attachments: ProblemAttachment[];
  subtasks: ProblemSubtask[];
  submissionPolicy: SubmissionPolicy;
  editorialPolicy: EditorialPolicy;
}

interface ProblemWorkspaceBootstrap {
  schemaVersion: 1;
  viewer: ViewerSummary | null;
  collection: ProblemCollectionPage;
  selected: ProblemDetail | null;
}
```

`statementHtml` must be sanitized. Collection summaries must not include statements, attachments,
subtasks, testcase data, or editorials.

## GET `/api/v1/problem-workspace`

Returns `ProblemWorkspaceBootstrap`.

| Parameter       | Type    | Default       | Rule                     |
| --------------- | ------- | ------------- | ------------------------ |
| `q`             | string  | empty         | Maximum 120 characters   |
| `status`        | enum    | `all`         | `ProblemStatus` or `all` |
| `types`         | CSV     | empty         | `ProblemType` values     |
| `tags`          | CSV     | empty         | OR matching              |
| `difficultyMin` | number  | `1`           | Inclusive                |
| `difficultyMax` | number  | `10`          | Inclusive                |
| `rated`         | boolean | `false`       | Exclude unrated problems |
| `sort`          | enum    | `solves-desc` | `ProblemSort`            |
| `limit`         | integer | `100`         | Maximum 256              |
| `selected`      | string  | empty         | Problem ID               |

- Without `selected`, `selected` in the response is `null`.
- With `selected`, the response includes that problem's detail.
- The first collection page includes `counts` and `tagFacets`.

## GET `/api/v1/problem-pages`

Returns `ProblemCollectionPage`.

First page accepts the same filter, sort, and limit parameters as the workspace endpoint.
Cursor pages accept `cursor` and `limit`.

Cursor pages:

- return only new `items` and `nextCursor`;
- omit `counts` and `tagFacets`;
- preserve filters, sort, catalog version, progress version, and the final sort key;
- return `409` with code `stale_cursor` when invalidated.

Default page size is 100. Maximum page size is 256.

## GET `/api/v1/problems/:id`

Returns `ProblemDetail`.

The endpoint knows:

- the problem from the `:id` path parameter;
- the user from the optional session cookie;
- the access context from the endpoint itself: this endpoint is practice access.

This endpoint is for normal practice access.

Return `200` when:

- the problem is published and public; or
- the authenticated user has explicit permission to view it.

Return `404 problem_not_found` when:

- the ID does not exist;
- the problem is draft, hidden, or deleted;
- the problem is contest-only;
- the user does not have permission;
- viewing it would reveal a contest problem before release.

For anonymous users viewing a public problem:

- `status` is `unattempted`;
- `score` is `null`;
- `submissionPolicy.allowed` is `false`;
- `submissionPolicy.blockedReason` is `login_required`.

Editorial rules:

- If available, `editorialPolicy.available` is `true` and `url` is present.
- If unavailable, `available` is `false`, `url` is `null`, and `reason` is set.
- Do not return editorial content or URLs before release.

Attachment rules:

- Return only attachments the user may download.
- Omit inaccessible attachments completely.
- Signed URLs include `expiresAt`.
- Non-expiring public URLs use `expiresAt: null`.

Contest-specific access will use:

```text
GET /api/v1/contests/:contestId/problems/:id
```

That endpoint receives both the contest ID and problem ID. Its response contract will be specified
with the contests frontend.

## Errors

Content type: `application/problem+json`.

```ts
interface ApiError {
  type: string;
  title: string;
  status: number;
  code: string;
  requestId: string;
}
```

Required codes:

| Status | Code                      |
| ------ | ------------------------- |
| 400    | `invalid_query`           |
| 401    | `unauthenticated`         |
| 404    | `problem_not_found`       |
| 409    | `stale_cursor`            |
| 429    | `rate_limited`            |
| 500    | `api_error`          |
| 503    | `temporarily_unavailable` |
