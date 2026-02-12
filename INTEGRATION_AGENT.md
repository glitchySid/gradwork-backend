# Gradwork Frontend Integration Spec — Build Guide for Agents

This document is a complete specification for building a frontend from scratch that integrates with the Gradwork backend. The backend is a **freelancing platform** built with Rust/Actix-Web. This spec defines exactly what the frontend should look like, what pages to build, what API calls to make, what types to use, and how every feature should work.

Read this entire document before writing any code.

---

## Table of Contents

1. [Platform Overview](#platform-overview)
2. [Recommended Tech Stack](#recommended-tech-stack)
3. [Environment Variables](#environment-variables)
4. [Authentication — How It Works](#authentication--how-it-works)
5. [TypeScript Types — Complete Definition](#typescript-types--complete-definition)
6. [API Client Setup](#api-client-setup)
7. [Backend API Reference — Every Endpoint](#backend-api-reference--every-endpoint)
8. [WebSocket Chat Protocol — Complete Spec](#websocket-chat-protocol--complete-spec)
9. [Database Schema — For Context](#database-schema--for-context)
10. [Pages and Routes — What to Build](#pages-and-routes--what-to-build)
11. [Component Architecture](#component-architecture)
12. [Auth Flow — Step by Step](#auth-flow--step-by-step)
13. [Role-Based UI Logic](#role-based-ui-logic)
14. [Chat Implementation Guide](#chat-implementation-guide)
15. [Authorization Rules — What the Backend Enforces](#authorization-rules--what-the-backend-enforces)
16. [Error Handling](#error-handling)
17. [Backend Known Issues — Frontend Must Handle](#backend-known-issues--frontend-must-handle)
18. [Deployment](#deployment)

---

## Platform Overview

Gradwork is a **freelancing platform** connecting two types of users:

- **Freelancers** post **gigs** (services they offer with a price) and showcase their work via **portfolios**
- **Clients** browse gigs and send **contract requests** to hire freelancers
- **Freelancers** accept or reject contract requests
- Once a contract is **accepted**, both parties can **chat in real-time** via WebSocket

### Domain Model

```
User (role: client | freelancer | admin)
  └── Freelancer posts Gigs
  └── Freelancer has Portfolios
  └── Client sends Contract requests on Gigs
  └── Freelancer accepts/rejects Contracts
  └── Accepted Contract → Chat (WebSocket + message history)
```

### Key Business Rules

- A client CANNOT send a contract request on their own gig
- A client can only have ONE contract per gig (unique constraint on `gig_id + user_id`)
- Only the gig owner (freelancer) can accept or reject contracts
- Only pending contracts can be accepted/rejected
- Only the client who created a contract can withdraw (delete) it, and only while pending
- Chat is ONLY available on accepted contracts
- Both the client and freelancer on an accepted contract can chat
- Users are auto-created on first authenticated API call (no separate registration endpoint)
- New users default to `client` role — they must call complete-profile to change to `freelancer`

---

## Recommended Tech Stack

| Layer | Technology | Notes |
|---|---|---|
| Framework | Next.js 14+ (App Router) | `src/app/` directory structure |
| Language | TypeScript (strict mode) | |
| Styling | Tailwind CSS + shadcn/ui | Use shadcn theme tokens consistently |
| Auth Client | `@supabase/supabase-js` | Must match backend's Supabase project |
| Data Fetching | TanStack React Query | For server state, caching, mutations |
| Forms | react-hook-form + zod | Schema validation |
| Icons | lucide-react | Matches shadcn defaults |
| WebSocket | Native browser `WebSocket` API | No library needed |
| Path Alias | `@/*` → `./src/*` | Standard Next.js alias |

---

## Environment Variables

```env
# Must match the backend's SUPABASE_URL (same Supabase project)
NEXT_PUBLIC_SUPABASE_URL=https://qngnzhaftgawnwohuolq.supabase.co

# Must match the backend's SUPABASE_ANON_KEY (same Supabase project)
NEXT_PUBLIC_SUPABASE_ANON_KEY=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

# Backend API URL — include /api suffix since all backend routes are under /api
# Local: http://localhost:8080/api
# Production: https://your-railway-app.up.railway.app/api
NEXT_PUBLIC_API_BASE_URL=http://localhost:8080/api

# For WebSocket connections (ws:// locally, wss:// in production)
# Local: ws://localhost:8080/api
# Production: wss://your-railway-app.up.railway.app/api
NEXT_PUBLIC_WS_BASE_URL=ws://localhost:8080/api
```

**CRITICAL:** The frontend and backend MUST use the same Supabase project. The backend validates JWTs using JWKS from its configured Supabase project. If the frontend uses a different Supabase project, every authenticated request will fail with a 401.

---

## Authentication — How It Works

### Overview

1. User authenticates with **Supabase** (email/password or OAuth)
2. Supabase issues a **JWT** (ES256, signed with EC keys)
3. Frontend sends this JWT as `Authorization: Bearer <token>` on every backend request
4. Backend validates the JWT via **JWKS** (fetches public keys from Supabase, cached 1hr)
5. Backend extracts `user_id` from `claims.sub` (UUID)
6. Backend calls `find_or_create_from_auth` — if user doesn't exist in DB, creates them with `role: client`
7. Backend returns the user model to the handler

### Key Implications for the Frontend

- **There is NO `/auth/register` endpoint.** Do NOT call one. The backend auto-creates users.
- After Supabase signup/login, call `GET /api/auth/me` to get/create the backend profile.
- New users are created with `role: "client"`. To become a freelancer, call `POST /api/auth/complete-profile` with `{ "role": "freelancer" }`.
- The JWT is in `session.access_token` from Supabase.
- For WebSocket connections, pass the token as a query parameter: `?token=<jwt>` (browsers don't support custom headers on WebSocket).

### JWT Claims Structure (for reference)

The backend decodes these fields from the Supabase JWT:

```
sub: string          → User UUID (used as primary key)
exp: number          → Expiration timestamp
email: string        → User's email
user_metadata:
  full_name: string  → Display name (from OAuth provider)
  avatar_url: string → Profile picture URL (from OAuth provider)
```

---

## TypeScript Types — Complete Definition

These types match EXACTLY what the backend sends and expects. Use these as your source of truth.

```typescript
// ============================================================
// Enums
// ============================================================

// Backend stores these as lowercase strings
type Role = "client" | "freelancer" | "admin";
type ContractStatus = "pending" | "accepted" | "rejected";

// ============================================================
// User
// ============================================================

// Response from: GET /api/auth/me, GET /api/users, GET /api/users/{id},
//                POST /api/auth/complete-profile, PUT /api/users/{id}
interface UserResponse {
  id: string;              // UUID
  email: string;
  username: string | null;
  display_name: string | null;
  avatar_url: string | null;
  role: Role;
  created_at: string;      // ISO 8601 datetime
  updated_at: string | null;
}

// Request body: POST /api/auth/complete-profile
interface CompleteProfileRequest {
  username?: string;
  role?: Role;
  display_name?: string;
  avatar_url?: string;
}

// Request body: PUT /api/users/{id}
interface UpdateUserRequest {
  email?: string;
  username?: string;
  display_name?: string;
  avatar_url?: string;
  role?: Role;
}

// ============================================================
// Gig
// ============================================================

// Response from: GET /api/gigs, GET /api/gigs/{id}, POST /api/gigs,
//                PUT /api/gigs/{id}, GET /api/gigs/user/{user_id}
interface Gig {
  id: string;              // UUID
  title: string;
  description: string;
  price: number;           // f64, e.g. 49.99
  user_id: string;         // UUID of the freelancer who posted it
  created_at: string;      // ISO 8601 datetime
}

// Request body: POST /api/gigs
interface CreateGigRequest {
  title: string;
  description: string;
  price: number;
}

// Request body: PUT /api/gigs/{id}
interface UpdateGigRequest {
  title?: string;
  description?: string;
  price?: number;
}

// ============================================================
// Portfolio
// ============================================================

// Response from: GET /api/portfolios, GET /api/portfolios/{id},
//                POST /api/portfolios, PUT /api/portfolios/{id},
//                GET /api/portfolios/freelancer/{freelancer_id}
interface Portfolio {
  id: string;              // UUID
  title: string;
  description: string;
  freelancer_id: string;   // UUID
  price: number;           // f64
  created_at: string;      // ISO 8601 datetime
}

// Request body: POST /api/portfolios
interface CreatePortfolioRequest {
  title: string;
  description: string;
  freelancer_id: string;   // Must match authenticated user's ID
  price: number;
}

// Request body: PUT /api/portfolios/{id}
interface UpdatePortfolioRequest {
  title?: string;
  description?: string;
  price?: number;
}

// ============================================================
// Contract
// ============================================================

// Response from: GET /api/contracts, GET /api/contracts/{id},
//                POST /api/contracts, PUT /api/contracts/{id}/status,
//                GET /api/contracts/gig/{gig_id},
//                GET /api/contracts/user/{user_id}
interface Contract {
  id: string;              // UUID
  gig_id: string;          // UUID
  user_id: string;         // UUID of the client who requested
  status: ContractStatus;
  created_at: string;      // ISO 8601 datetime
}

// Request body: POST /api/contracts
interface CreateContractRequest {
  gig_id: string;          // UUID — user_id is set from auth automatically
}

// Request body: PUT /api/contracts/{id}/status
interface UpdateContractStatusRequest {
  status: ContractStatus;  // "accepted" or "rejected"
}

// ============================================================
// Messages & Chat (REST)
// ============================================================

// Response from: GET /api/chat/{contract_id}/messages,
//                PUT /api/chat/messages/{id}/read
interface MessageResponse {
  id: string;              // UUID
  contract_id: string;     // UUID
  sender_id: string;       // UUID
  content: string;
  is_read: boolean;
  created_at: string;      // ISO 8601 datetime
}

// Query params: GET /api/chat/{contract_id}/messages
interface MessageQuery {
  page?: number;           // Default 1, min 1
  limit?: number;          // Default 50, max 100
}

// Response from: GET /api/chat/conversations
interface ConversationSummary {
  contract_id: string;     // UUID
  other_user_id: string;   // UUID
  other_user_name: string | null;
  last_message: string | null;
  last_message_at: string | null;  // ISO 8601 datetime
  unread_count: number;
}

// ============================================================
// WebSocket Messages
// ============================================================

// Client → Server (send via ws.send(JSON.stringify(...)))
type ClientWsMessage =
  | { type: "send_message"; content: string }
  | { type: "mark_read"; message_id: string }
  | { type: "typing" }
  | { type: "stop_typing" };

// Server → Client (received via ws.onmessage)
type ServerWsMessage =
  | { type: "new_message"; id: string; sender_id: string; content: string; created_at: string }
  | { type: "message_read"; message_id: string }
  | { type: "user_typing"; user_id: string }
  | { type: "user_stop_typing"; user_id: string }
  | { type: "presence"; user_id: string; online: boolean }
  | { type: "error"; message: string };

// ============================================================
// Generic API response for delete operations
// ============================================================

interface DeleteResponse {
  message: string;
}
```

---

## API Client Setup

```typescript
const API_BASE_URL = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:8080/api";

interface RequestOptions {
  method?: string;
  body?: unknown;
  token?: string;
}

export async function apiClient<T>(
  endpoint: string,
  options: RequestOptions = {}
): Promise<T> {
  const { method = "GET", body, token } = options;
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (token) {
    headers["Authorization"] = `Bearer ${token}`;
  }

  const res = await fetch(`${API_BASE_URL}${endpoint}`, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  });

  if (!res.ok) {
    const error = await res.json().catch(() => ({ message: "An error occurred" }));
    throw new Error(error.message || `Request failed with status ${res.status}`);
  }

  // Handle 204 No Content
  if (res.status === 204) {
    return undefined as T;
  }

  return res.json();
}
```

**Endpoint format:** Always start with `/` (e.g., `/auth/me`, `/gigs`, `/contracts`). The base URL already includes `/api`.

---

## Backend API Reference — Every Endpoint

All endpoints are prefixed with `/api` (included in `API_BASE_URL`). So in the frontend, call them as `/auth/me`, `/gigs`, etc.

### Auth

| Method | Endpoint | Auth | Body | Response | Notes |
|---|---|---|---|---|---|
| `GET` | `/auth/me` | Bearer | — | `UserResponse` | Returns current user. Auto-creates on first call. |
| `POST` | `/auth/complete-profile` | Bearer | `CompleteProfileRequest` | `UserResponse` | Update profile fields (role, username, display_name, avatar_url). All fields optional. |

### Users

| Method | Endpoint | Auth | Body | Response | Notes |
|---|---|---|---|---|---|
| `GET` | `/users` | Bearer | — | `UserResponse[]` | List all users |
| `GET` | `/users/{id}` | Bearer | — | `UserResponse` | Get user by ID. 404 if not found. |
| `PUT` | `/users/{id}` | Bearer | `UpdateUserRequest` | `UserResponse` | Self-only. 403 if `id` != auth user. |
| `DELETE` | `/users/{id}` | Bearer | — | `DeleteResponse` | Self-only. 403 if `id` != auth user. |

### Gigs

| Method | Endpoint | Auth | Body | Response | Notes |
|---|---|---|---|---|---|
| `GET` | `/gigs` | **NONE** | — | `Gig[]` | Public. No auth required. Returns all gigs. |
| `POST` | `/gigs` | Bearer | `CreateGigRequest` | `Gig` | `user_id` set from auth token automatically. Status 201. |
| `GET` | `/gigs/{id}` | Bearer | — | `Gig` | 404 if not found. |
| `PUT` | `/gigs/{id}` | Bearer | `UpdateGigRequest` | `Gig` | WARNING: No ownership check on backend (see Known Issues). |
| `DELETE` | `/gigs/{id}` | Bearer | — | `DeleteResponse` | WARNING: No ownership check on backend. |
| `GET` | `/gigs/user/{user_id}` | Bearer | — | `Gig[]` | Get all gigs posted by a specific user. |
| `DELETE` | `/gigs/user/{user_id}` | Bearer | — | (204) | Delete all gigs by a user. |

### Portfolios

| Method | Endpoint | Auth | Body | Response | Notes |
|---|---|---|---|---|---|
| `GET` | `/portfolios` | Bearer | — | `Portfolio[]` | List all portfolio items. |
| `POST` | `/portfolios` | Bearer | `CreatePortfolioRequest` | `Portfolio` | `freelancer_id` must match auth user. 403 otherwise. Status 201. |
| `GET` | `/portfolios/{id}` | Bearer | — | `Portfolio` | 404 if not found. |
| `PUT` | `/portfolios/{id}` | Bearer | `UpdatePortfolioRequest` | `Portfolio` | Owner only. 403 if not owner. |
| `DELETE` | `/portfolios/{id}` | Bearer | — | `DeleteResponse` | Owner only. 403 if not owner. |
| `GET` | `/portfolios/freelancer/{freelancer_id}` | Bearer | — | `Portfolio[]` | Get all portfolio items for a freelancer. |

### Contracts

| Method | Endpoint | Auth | Body | Response | Notes |
|---|---|---|---|---|---|
| `GET` | `/contracts` | Bearer | — | `Contract[]` | All contracts where user is client OR freelancer (gig owner). Merged + deduplicated. |
| `POST` | `/contracts` | Bearer | `CreateContractRequest` | `Contract` | `user_id` auto-set from auth. 400 if own gig. 409 if duplicate. 404 if gig not found. Status 201. |
| `GET` | `/contracts/{id}` | Bearer | — | `Contract` | Must be party to contract. 403 otherwise. |
| `DELETE` | `/contracts/{id}` | Bearer | — | `DeleteResponse` | Client-only + pending-only. 403/400 otherwise. |
| `PUT` | `/contracts/{id}/status` | Bearer | `UpdateContractStatusRequest` | `Contract` | Gig-owner-only + pending-only. 403/400 otherwise. |
| `GET` | `/contracts/gig/{gig_id}` | Bearer | — | `Contract[]` | Gig-owner-only. 403 otherwise. |
| `GET` | `/contracts/user/{user_id}` | Bearer | — | `Contract[]` | Self-only. 403 otherwise. |

### Chat

| Method | Endpoint | Auth | Body | Response | Notes |
|---|---|---|---|---|---|
| `GET` | `/chat/ws/{contract_id}?token=<jwt>` | Query param | — | WebSocket | Contract must be accepted. User must be party. |
| `GET` | `/chat/conversations` | Bearer | — | `ConversationSummary[]` | Sorted by `last_message_at` DESC. Only accepted contracts. |
| `GET` | `/chat/{contract_id}/messages` | Bearer | Query: `?page=1&limit=50` | `MessageResponse[]` | Paginated. Ordered by `created_at DESC`. Max limit 100. |
| `PUT` | `/chat/messages/{id}/read` | Bearer | — | `MessageResponse` | Marks a single message as read. |

---

## WebSocket Chat Protocol — Complete Spec

### Connecting

```typescript
const WS_BASE_URL = process.env.NEXT_PUBLIC_WS_BASE_URL || "ws://localhost:8080/api";

function connectChat(contractId: string, token: string): WebSocket {
  const ws = new WebSocket(`${WS_BASE_URL}/chat/ws/${contractId}?token=${token}`);
  return ws;
}
```

### Prerequisites

- Contract must have `status === "accepted"`
- Authenticated user must be either the client (`contract.user_id`) or the freelancer (gig owner)
- If either condition fails, the WebSocket upgrade is rejected (HTTP error before upgrade)

### Client → Server Messages

Send JSON via `ws.send(JSON.stringify(msg))`:

```typescript
// Send a chat message
ws.send(JSON.stringify({ type: "send_message", content: "Hello!" }));

// Mark a message as read
ws.send(JSON.stringify({ type: "mark_read", message_id: "uuid-here" }));

// Signal that you're typing
ws.send(JSON.stringify({ type: "typing" }));

// Signal that you stopped typing
ws.send(JSON.stringify({ type: "stop_typing" }));
```

### Server → Client Messages

Handle via `ws.onmessage`:

| Type | Fields | Sent To | Description |
|---|---|---|---|
| `new_message` | `id`, `sender_id`, `content`, `created_at` | ALL in room (including sender) | A new message was sent. Already persisted to DB. |
| `message_read` | `message_id` | ALL in room | A message was marked as read. |
| `user_typing` | `user_id` | ALL except typer | Someone started typing. |
| `user_stop_typing` | `user_id` | ALL except typer | Someone stopped typing. |
| `presence` | `user_id`, `online` | ALL in room | Someone joined (`online: true`) or left (`online: false`). |
| `error` | `message` | Sender only | An error occurred processing the client's message. |

### Important Behavior Notes

1. **`new_message` is sent to the sender too.** The frontend should use the server-echoed message as the source of truth (it includes the server-assigned `id` and `created_at`). You can show an optimistic message immediately, then replace it when the server echo arrives.

2. **Messages are ordered DESC from the REST endpoint** (`created_at DESC`). When loading history, reverse the array for chronological display, or paginate from the last page.

3. **Presence events** fire on connect/disconnect. If a user has multiple tabs open, `online: false` only fires when ALL their connections close.

4. **Typing indicators** should be debounced. Send `typing` when the user starts typing, `stop_typing` after ~2 seconds of no keystrokes. The backend does not auto-expire typing state — the frontend must send `stop_typing`.

5. **The WebSocket connection stays alive** via ping/pong (handled automatically by the browser and backend). If the connection drops, the frontend should attempt reconnection with exponential backoff.

---

## Database Schema — For Context

Understanding the DB helps when reasoning about what data is available.

### Entity Relationships

```
users ──< gigs            (user_id → users.id)
users ──< contracts        (user_id → users.id, the "client")
users ──< portfolios       (freelancer_id → users.id)
users ──< messages         (sender_id → users.id)
gigs  ──< contracts        (gig_id → gigs.id)
contracts ──< messages     (contract_id → contracts.id)
```

A contract connects a **client** (`contracts.user_id`) to a **freelancer** (the owner of the gig, found via `gigs.user_id` through `contracts.gig_id`).

### Tables

**users:** `id` (UUID PK), `email` (unique), `username` (nullable, unique), `role` ("client"/"freelancer"/"admin"), `display_name`, `avatar_url`, `auth_provider` (default "google"), `created_at`, `updated_at`

**gigs:** `id` (UUID PK), `title`, `description` (text), `price` (double), `user_id` (FK → users), `created_at`

**contracts:** `id` (UUID PK), `gig_id` (FK → gigs), `user_id` (FK → users), `status` ("pending"/"accepted"/"rejected"), `created_at`. UNIQUE index on `(gig_id, user_id)`.

**portfolios:** `id` (UUID PK), `title`, `description` (text), `freelancer_id` (FK → users), `price` (double), `created_at`

**messages:** `id` (UUID PK), `contract_id` (FK → contracts), `sender_id` (FK → users), `content` (text), `is_read` (boolean, default false), `created_at`. Index on `(contract_id, created_at)`.

---

## Pages and Routes — What to Build

### Public Routes (no auth required)

| Route | Page | Description |
|---|---|---|
| `/` | Landing Page | Marketing page. Hero, features, CTAs to login/register. |
| `/login` | Login | Email/password + optional OAuth (Google). |
| `/register` | Register | Email/password signup with role selection (client/freelancer). |
| `/auth/callback` | Auth Callback | Handles Supabase OAuth/email verification redirects. |
| `/gigs` | Browse Gigs | Public listing of all gigs. Calls `GET /gigs` (no auth). |

### Protected Routes — All Users

| Route | Page | Description |
|---|---|---|
| `/dashboard` | Dashboard | Role-based dashboard home. Quick actions and stats. |
| `/profile` | Profile | Edit own profile (username, display_name, avatar_url, role). |
| `/gigs/{id}` | Gig Detail | View gig details. Clients see "Request Contract" button. Freelancers see their own gig's contracts. |
| `/contracts` | My Contracts | List all contracts (as client and as freelancer). Status badges (pending/accepted/rejected). |
| `/chat` | Conversations | List of accepted contracts with chat available. Shows unread counts, last message preview. |
| `/chat/{contractId}` | Chat Room | Real-time chat for an accepted contract. Message history, typing indicators, presence. |
| `/complete-profile` | Complete Profile | Post-signup role/username selection. Redirect here if profile is incomplete. |

### Protected Routes — Freelancer Only

| Route | Page | Description |
|---|---|---|
| `/my-gigs` | My Gigs | List of gigs posted by current freelancer. Create/edit/delete. |
| `/my-gigs/new` | Create Gig | Form: title, description, price. |
| `/my-gigs/{id}/edit` | Edit Gig | Pre-filled form for editing a gig. |
| `/portfolio` | My Portfolio | List of portfolio items. Create/edit/delete. |
| `/portfolio/new` | Add Portfolio Item | Form: title, description, price. |
| `/portfolio/{id}/edit` | Edit Portfolio Item | Pre-filled form. |
| `/contracts/gig/{gigId}` | Contracts on Gig | View all contract requests on a specific gig. Accept/reject buttons. |

### Protected Routes — Client Only

| Route | Page | Description |
|---|---|---|
| `/my-contracts` | My Contract Requests | Contracts the client has sent. Withdraw pending ones. |

### Route Group Structure (suggested)

```
src/app/
├── page.tsx                          # Landing (public)
├── layout.tsx                        # Root layout: fonts, Providers
├── (auth)/
│   ├── login/page.tsx
│   ├── register/page.tsx
│   └── complete-profile/page.tsx
├── auth/callback/page.tsx            # Not in (auth) group — different layout
├── (public)/
│   └── gigs/page.tsx                 # Public gig browsing
└── (dashboard)/
    ├── layout.tsx                    # Auth guard, sidebar, main content area
    ├── dashboard/page.tsx
    ├── profile/page.tsx
    ├── gigs/[id]/page.tsx            # Gig detail (auth required for contract actions)
    ├── contracts/page.tsx            # All my contracts
    ├── contracts/gig/[gigId]/page.tsx # Contracts on a specific gig (freelancer)
    ├── chat/page.tsx                 # Conversation list
    ├── chat/[contractId]/page.tsx    # Chat room
    ├── my-gigs/page.tsx              # Freelancer's gigs
    ├── my-gigs/new/page.tsx          # Create gig
    ├── my-gigs/[id]/edit/page.tsx    # Edit gig
    ├── portfolio/page.tsx            # Freelancer's portfolio
    ├── portfolio/new/page.tsx        # Add portfolio item
    └── portfolio/[id]/edit/page.tsx  # Edit portfolio item
```

---

## Component Architecture

### Layout Components

| Component | Purpose |
|---|---|
| `RootLayout` | HTML head, fonts (Geist), `<Providers>` wrapper |
| `Providers` | `QueryClientProvider` + `AuthProvider` |
| `DashboardLayout` | Auth guard + `<Sidebar>` + `<main>` content. Redirects to `/login` if no session, `/complete-profile` if profile incomplete. |
| `Sidebar` | Role-based navigation links with icons |

### Auth Components

| Component | Purpose |
|---|---|
| `AuthProvider` | React context: `user`, `session`, `profile`, `loading`, `signOut`, `refreshProfile` |
| `LoginForm` | Email/password + OAuth buttons |
| `RegisterForm` | Role selector + email/password + name |
| `CompleteProfileForm` | Username + role selector (for post-signup) |

### Gig Components

| Component | Purpose |
|---|---|
| `GigCard` | Card showing title, description snippet, price, freelancer name |
| `GigGrid` / `GigList` | Grid/list layout of `GigCard`s |
| `GigForm` | Create/edit form: title, description, price |
| `GigDetail` | Full gig view with contract request button (for clients) |

### Contract Components

| Component | Purpose |
|---|---|
| `ContractCard` | Shows contract with gig info, status badge, other party name |
| `ContractList` | List of contracts with status filter tabs (all/pending/accepted/rejected) |
| `ContractActions` | Accept/Reject buttons (freelancer) or Withdraw button (client, pending only) |
| `StatusBadge` | Color-coded badge: pending (yellow), accepted (green), rejected (red) |

### Portfolio Components

| Component | Purpose |
|---|---|
| `PortfolioCard` | Card showing title, description, price |
| `PortfolioGrid` | Grid layout of portfolio items |
| `PortfolioForm` | Create/edit form: title, description, price |

### Chat Components

| Component | Purpose |
|---|---|
| `ConversationList` | List of conversations with other user name, last message, unread badge, timestamp |
| `ChatRoom` | Full chat UI: message list, input box, typing indicator, online status |
| `MessageBubble` | Single message: content, timestamp, read receipt. Left/right aligned based on sender. |
| `TypingIndicator` | "User is typing..." animation |
| `OnlineStatus` | Green/gray dot for online/offline |
| `ChatInput` | Text input + send button. Emits typing/stop_typing events. |

### Shared UI Components (shadcn/ui)

Use shadcn components for consistent styling: `Button`, `Card`, `Input`, `Textarea`, `Select`, `Label`, `Badge`, `Avatar`, `Tabs`, `Separator`, `Dialog`, `DropdownMenu`, `Skeleton` (loading states), `Toast` (notifications).

---

## Auth Flow — Step by Step

### Registration

```
1. User navigates to /register
2. User selects role: "client" or "freelancer"
3. User fills: email, password, full name
4. Frontend calls: supabase.auth.signUp({ email, password, options: { data: { full_name } } })
5. If Supabase requires email confirmation:
   → Show "Check your email" message
   → User clicks link → redirected to /auth/callback
   → /auth/callback page calls supabase.auth.getSession()
   → On session: call GET /auth/me → backend auto-creates user
   → Call POST /auth/complete-profile with { role, display_name: full_name }
   → Redirect to /dashboard
6. If Supabase returns session immediately (email confirm disabled):
   → Call GET /auth/me → backend auto-creates user
   → Call POST /auth/complete-profile with { role, display_name: full_name }
   → Redirect to /dashboard
```

### Login

```
1. User navigates to /login
2. User fills: email, password
3. Frontend calls: supabase.auth.signInWithPassword({ email, password })
4. On success: session available
5. Call GET /auth/me to fetch backend profile
6. Redirect to /dashboard
```

### OAuth (Google)

```
1. User clicks "Sign in with Google"
2. Frontend calls: supabase.auth.signInWithOAuth({ provider: 'google', options: { redirectTo: '/auth/callback' } })
3. User completes OAuth flow → redirected to /auth/callback
4. /auth/callback: supabase.auth.getSession() → session available
5. Call GET /auth/me (auto-creates user with display_name and avatar_url from Google metadata)
6. If user.role is "client" and hasn't completed profile → redirect to /complete-profile
7. Otherwise → redirect to /dashboard
```

### Session Management (AuthProvider)

```
1. On mount: supabase.auth.getSession()
2. Subscribe to: supabase.auth.onAuthStateChange
3. On SIGNED_IN / TOKEN_REFRESHED with session:
   → Extract session.access_token
   → Call GET /auth/me with token
   → Store result as "profile"
4. On SIGNED_OUT:
   → Clear user, session, profile
5. Provide via context: { user, session, profile, loading, signOut, refreshProfile }
```

### Profile Completeness Check

A profile is "incomplete" if any of these are true:
- `username` is null
- `role` is still the default `"client"` and user intended to be a freelancer

The dashboard layout should check this and redirect to `/complete-profile` if needed.

---

## Role-Based UI Logic

### Sidebar Navigation

**Client:**
- Dashboard (`/dashboard`)
- Browse Gigs (`/gigs`)
- My Contracts (`/contracts`)
- Messages (`/chat`)
- Profile (`/profile`)

**Freelancer:**
- Dashboard (`/dashboard`)
- My Gigs (`/my-gigs`)
- My Portfolio (`/portfolio`)
- Contracts (`/contracts`)
- Messages (`/chat`)
- Profile (`/profile`)

**Admin:**
- Same as freelancer + admin tools (future)

### Gig Detail Page — Role-Based Actions

- **Client viewing someone else's gig:** Show "Request Contract" button (if no existing contract for this gig). If contract already exists, show its status.
- **Freelancer viewing their own gig:** Show "Edit" and "Delete" buttons. Show list of contract requests with accept/reject actions.
- **Freelancer viewing another freelancer's gig:** Read-only view (no contract request since freelancers don't hire).

### Dashboard Cards

**Client:**
- "Browse Gigs" — link to `/gigs`
- "My Contracts" — count of pending/accepted contracts, link to `/contracts`
- "Messages" — unread count, link to `/chat`

**Freelancer:**
- "My Gigs" — count of active gigs, link to `/my-gigs`
- "Contract Requests" — count of pending requests, link to `/contracts`
- "Messages" — unread count, link to `/chat`
- "Portfolio" — count of items, link to `/portfolio`

---

## Chat Implementation Guide

### Architecture

```
ConversationList page (/chat)
  └── Fetches: GET /chat/conversations
  └── Displays: list of ConversationSummary items
  └── Click → navigate to /chat/{contractId}

ChatRoom page (/chat/{contractId})
  └── On mount:
      ├── Fetch message history: GET /chat/{contractId}/messages
      ├── Open WebSocket: ws://host/api/chat/ws/{contractId}?token=...
      └── Track connection state
  └── On message from WS:
      ├── new_message → append to message list
      ├── message_read → update read status
      ├── user_typing → show typing indicator
      ├── user_stop_typing → hide typing indicator
      ├── presence → update online status
      └── error → show error toast
  └── On send:
      ├── Optimistically add message to list
      ├── ws.send({ type: "send_message", content: "..." })
      └── Replace optimistic message with server echo (match by content + timing, or just append and deduplicate by id)
  └── On unmount:
      └── ws.close()
```

### Custom Hook: `useChat`

```typescript
function useChat(contractId: string, token: string, currentUserId: string) {
  // State:
  //   messages: MessageResponse[]
  //   isConnected: boolean
  //   otherUserTyping: boolean
  //   otherUserOnline: boolean

  // On mount:
  //   1. Fetch history via REST (GET /chat/{contractId}/messages)
  //   2. Open WebSocket connection
  //   3. Handle all ServerWsMessage types
  //   4. Return { messages, isConnected, otherUserTyping, otherUserOnline, sendMessage, sendTyping, sendStopTyping }

  // Reconnection: if WebSocket closes unexpectedly, retry with exponential backoff (1s, 2s, 4s, max 30s)
}
```

### Message Display Logic

- Messages from `sender_id === currentUserId` → right-aligned, primary color
- Messages from other user → left-aligned, muted color
- Show timestamp on each message (relative: "2m ago", or absolute if older than 24h)
- Show read receipt (checkmark) on own messages where `is_read === true`
- Group consecutive messages from same sender (no repeated avatar/name)

### Typing Indicator Logic

- On keystroke in chat input: send `{ type: "typing" }` (debounce — don't send more than once per second)
- After 2 seconds of no typing: send `{ type: "stop_typing" }`
- On receiving `user_typing`: show "typing..." under the message list
- On receiving `user_stop_typing`: hide indicator
- Clear typing indicator when a `new_message` arrives from the typing user

### Loading History with Pagination

- Load most recent page first (page 1, limit 50)
- When user scrolls to top, load next page (page 2, etc.)
- Reverse the array from each page for chronological display (API returns DESC)
- Prepend older messages to the top of the list
- Preserve scroll position when prepending

---

## Authorization Rules — What the Backend Enforces

The frontend should reflect these rules in the UI (hide buttons, show disabled states) but the backend enforces them regardless.

| Action | Who | Condition | Error |
|---|---|---|---|
| View/create gigs | Any authenticated user | — | — |
| Edit/delete gig | Any authenticated user | No ownership check (backend bug) | — |
| Create portfolio | Freelancer | `freelancer_id` must match auth user | 403 |
| Edit/delete portfolio | Freelancer | Must own the portfolio item | 403 |
| Create contract | Client | Cannot be own gig. One per gig per user. | 400 / 409 |
| View contract | Client or freelancer | Must be party to the contract | 403 |
| Accept/reject contract | Freelancer (gig owner) | Contract must be pending | 403 / 400 |
| Withdraw contract | Client (contract creator) | Contract must be pending | 403 / 400 |
| View contracts by gig | Freelancer (gig owner) | Must own the gig | 403 |
| View contracts by user | Self | `user_id` must match auth user | 403 |
| Open WebSocket chat | Client or freelancer | Contract must be accepted | Rejected before upgrade |
| View/send messages | Client or freelancer | Contract must be accepted | 403 |
| Update own profile | Self | `id` must match auth user | 403 |
| Delete own account | Self | `id` must match auth user | 403 |

---

## Error Handling

The backend returns errors as JSON:

```json
{ "message": "Error description here" }
```

Or sometimes just a string body. HTTP status codes used:

| Status | Meaning |
|---|---|
| 200 | Success |
| 201 | Created (POST success) |
| 204 | No Content (delete all gigs by user) |
| 400 | Bad request (e.g., contract on own gig, non-pending contract) |
| 401 | Unauthorized (invalid/missing JWT) |
| 403 | Forbidden (valid JWT but not authorized for this action) |
| 404 | Not found |
| 409 | Conflict (duplicate contract) |
| 500 | Internal server error |

### Frontend Error Handling Strategy

- **401:** Token expired or invalid → redirect to `/login`, clear session
- **403:** Show "Not authorized" message or hide the action in UI
- **404:** Show "Not found" page or message
- **409:** Show "You already sent a contract request for this gig"
- **400:** Show the error message from the response
- **500:** Show generic "Something went wrong" with retry option

---

## Backend Known Issues — Frontend Must Handle

1. **No gig ownership check on edit/delete.** The backend allows ANY authenticated user to edit/delete any gig. The frontend should ONLY show edit/delete buttons when `gig.user_id === currentUser.id`, but know that the backend won't stop someone from doing it via API.

2. **Default role is always `"client"`.** When a new user is auto-created, they get `role: "client"`. If a user signs up intending to be a freelancer, the frontend must immediately call `POST /auth/complete-profile` with `{ "role": "freelancer" }` after the initial `GET /auth/me`.

3. **`auth_provider` is always `"google"`.** Even for email/password signups. This is cosmetic and doesn't affect functionality.

4. **Messages are returned in DESC order.** The `GET /chat/{contract_id}/messages` endpoint returns newest first. The frontend must reverse the array for chronological display in the chat UI.

5. **`new_message` echoed to sender.** The WebSocket broadcasts new messages to ALL room members including the sender. The frontend should either:
   - Use the server echo as the authoritative message (replace optimistic with server version by matching `id`)
   - Or deduplicate by `id` to avoid showing the same message twice

6. **No input validation on backend.** The backend doesn't validate empty strings, negative prices, max lengths, etc. The frontend MUST validate inputs before sending:
   - Title: non-empty, reasonable max length (e.g., 200 chars)
   - Description: non-empty
   - Price: positive number
   - Username: non-empty if provided, no spaces
   - Message content: non-empty (backend rejects empty messages in WS handler)

7. **No pagination on gigs, contracts, portfolios.** Only the messages endpoint supports pagination. For now, all items are returned in a single array. This is fine for small datasets but may need backend changes later.

8. **No search or filtering on any endpoint.** The frontend can implement client-side filtering/search on the returned arrays.

---

## Deployment

### Frontend (Vercel recommended)

- Standard Next.js deployment
- Set environment variables in Vercel dashboard:
  - `NEXT_PUBLIC_SUPABASE_URL` — same as backend
  - `NEXT_PUBLIC_SUPABASE_ANON_KEY` — same as backend
  - `NEXT_PUBLIC_API_BASE_URL` — Railway backend URL with `/api` (e.g., `https://gradwork-backend.up.railway.app/api`)
  - `NEXT_PUBLIC_WS_BASE_URL` — Railway backend URL with `wss://` and `/api` (e.g., `wss://gradwork-backend.up.railway.app/api`)

### Backend (Railway)

- Already configured to bind to `0.0.0.0:$PORT`
- CORS: currently `allow_any_origin()` — after deployment, should be tightened to allow only the Vercel frontend URL
- Required env vars in Railway: `DATABASE_URL`, `SUPABASE_URL`, `SUPABASE_ANON_KEY`, optionally `NIXPACKS_RUST_VERSION=1.88.0`
- The `messages` table migration must be run on the production DB

### CORS Note

The backend currently allows all origins. Once the frontend URL is known (e.g., `https://gradwork.vercel.app`), the backend's `main.rs` should be updated:

```rust
let cors = Cors::default()
    .allowed_origin("https://gradwork.vercel.app")
    .allowed_origin("http://localhost:3000")  // local dev
    // ... rest of config
```
