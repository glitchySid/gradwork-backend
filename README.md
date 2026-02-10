# Gradwork Backend

A Rust backend for the Gradwork freelancing platform, built with Actix-Web, SeaORM, and Supabase Auth.

## Prerequisites

- [Rust](https://rustup.rs/) (edition 2024, stable toolchain)
- PostgreSQL database
- [Supabase](https://supabase.com/) project (for authentication)

## Setup

### 1. Clone the repository

```bash
git clone <repo-url>
cd gradwork-backend
```

### 2. Configure environment variables

Create a `.env` file in the project root:

```env
DATABASE_URL=postgres://user:password@localhost:5432/gradwork
SUPABASE_URL=https://YOUR_PROJECT_REF.supabase.co
SUPABASE_ANON_KEY=your-supabase-anon-key
```

- `DATABASE_URL` -- connection string for your Postgres database.
- `SUPABASE_URL` -- your Supabase project URL (format: `https://<project-ref>.supabase.co`).
- `SUPABASE_ANON_KEY` -- the `anon` (public) key from your Supabase project settings > API.

### 3. Run database migrations

```bash
cargo run -p migration -- up
```

To rollback the last migration:

```bash
cargo run -p migration -- down
```

To check migration status:

```bash
cargo run -p migration -- status
```

### 4. Start the server

```bash
cargo run
```

The server starts at `http://127.0.0.1:8080`.

---

## Authentication

All API routes (except `GET /api/gigs`) require a valid Supabase JWT in the `Authorization` header.

### Getting a token via Supabase

#### Option A: Google OAuth (recommended)

1. Enable Google provider in your Supabase dashboard: **Authentication > Providers > Google**.
2. From your frontend, initiate the OAuth flow using the Supabase client SDK:

```javascript
import { createClient } from '@supabase/supabase-js'

const supabase = createClient('https://YOUR_PROJECT_REF.supabase.co', 'your-anon-key')

// Sign in with Google
const { data, error } = await supabase.auth.signInWithOAuth({
  provider: 'google',
})

// After redirect, get the session token
const { data: { session } } = await supabase.auth.getSession()
const token = session.access_token
```

#### Option B: Email/password (for testing)

1. Enable Email provider in Supabase dashboard: **Authentication > Providers > Email**.
2. Sign up / sign in:

```bash
# Sign up
curl -X POST 'https://YOUR_PROJECT_REF.supabase.co/auth/v1/signup' \
  -H 'apikey: your-anon-key' \
  -H 'Content-Type: application/json' \
  -d '{"email": "test@example.com", "password": "your-password"}'

# Sign in (returns access_token)
curl -X POST 'https://YOUR_PROJECT_REF.supabase.co/auth/v1/token?grant_type=password' \
  -H 'apikey: your-anon-key' \
  -H 'Content-Type: application/json' \
  -d '{"email": "test@example.com", "password": "your-password"}'
```

The sign-in response includes an `access_token`. Use it as the Bearer token for all API requests.

### Using the token

Include the token in every request:

```
Authorization: Bearer <your-access-token>
```

The first time a user makes an authenticated request, the backend automatically creates a user record in the `users` table from the JWT claims.

---

## API Routes

Base URL: `http://127.0.0.1:8080/api`

All routes return JSON. Errors follow the format:

```json
{ "error": "Error description" }
```

---

### Auth

#### `GET /api/auth/me`

Returns the currently authenticated user's profile.

**Headers:** `Authorization: Bearer <token>`

**Response (200):**

```json
{
  "id": "uuid",
  "email": "user@example.com",
  "username": null,
  "display_name": "John Doe",
  "avatar_url": "https://...",
  "role": "client",
  "created_at": "2025-02-06T00:00:00Z",
  "updated_at": null
}
```

---

#### `POST /api/auth/complete-profile`

Set username, role, and display name after first login.

**Headers:** `Authorization: Bearer <token>`

**Body:**

```json
{
  "username": "johndoe",
  "role": "freelancer",
  "display_name": "John Doe"
}
```

All fields are optional. Valid roles: `"client"`, `"freelancer"`, `"admin"`.

**Response (200):** Updated user object.

---

### Users

#### `GET /api/users`

List all users.

**Headers:** `Authorization: Bearer <token>`

**Response (200):** Array of user objects.

---

#### `GET /api/users/{id}`

Get a single user by ID.

**Headers:** `Authorization: Bearer <token>`

**Response (200):** User object.
**Response (404):** `{ "error": "User {id} not found" }`

---

#### `PUT /api/users/{id}`

Update a user. Users can only update their own account.

**Headers:** `Authorization: Bearer <token>`

**Body:**

```json
{
  "email": "new@example.com",
  "username": "newusername",
  "display_name": "New Name",
  "avatar_url": "https://...",
  "role": "freelancer"
}
```

All fields are optional.

**Response (200):** Updated user object.
**Response (403):** `{ "error": "You can only update your own account" }`

---

#### `DELETE /api/users/{id}`

Delete a user. Users can only delete their own account.

**Headers:** `Authorization: Bearer <token>`

**Response (200):** `{ "message": "User {id} deleted" }`
**Response (403):** `{ "error": "You can only delete your own account" }`

---

### Gigs

#### `GET /api/gigs`

List all gigs. Does **not** require authentication.

**Response (200):** Array of gig objects.

```json
[
  {
    "id": "uuid",
    "title": "Build a website",
    "description": "Full-stack web development",
    "price": 500.0,
    "user_id": "uuid",
    "created_at": "2025-02-06T00:00:00Z"
  }
]
```

---

#### `GET /api/gigs/{id}`

Get a single gig by ID.

**Headers:** `Authorization: Bearer <token>`

**Response (200):** Gig object.
**Response (404):** `{ "error": "Gig {id} not found" }`

---

#### `GET /api/gigs/user/{user_id}`

Get all gigs created by a specific user.

**Headers:** `Authorization: Bearer <token>`

**Response (200):** Array of gig objects.

---

#### `POST /api/gigs`

Create a new gig. The `user_id` is automatically set from the authenticated user's JWT.

**Headers:** `Authorization: Bearer <token>`

**Body:**

```json
{
  "title": "Build a website",
  "description": "Full-stack web development project",
  "price": 500.0
}
```

**Response (201):** Created gig object.

---

#### `PUT /api/gigs/{id}`

Update a gig.

**Headers:** `Authorization: Bearer <token>`

**Body:**

```json
{
  "title": "Updated title",
  "description": "Updated description",
  "price": 750.0
}
```

All fields are optional.

**Response (200):** Updated gig object.
**Response (404):** `{ "error": "Failed to update gig: Gig not found" }`

---

#### `DELETE /api/gigs/{id}`

Delete a gig by ID.

**Headers:** `Authorization: Bearer <token>`

**Response (200):** `{ "message": "Gig {id} deleted" }`
**Response (404):** `{ "error": "Gig {id} not found" }`

---

#### `DELETE /api/gigs/user/{user_id}`

Delete all gigs by a specific user.

**Headers:** `Authorization: Bearer <token>`

**Response (204):** No content.

---

### Portfolios

#### `GET /api/portfolios`

List all portfolio items.

**Headers:** `Authorization: Bearer <token>`

**Response (200):** Array of portfolio objects.

```json
[
  {
    "id": "uuid",
    "title": "My Project",
    "description": "A project I built",
    "freelancer_id": "uuid",
    "price": 300.0,
    "created_at": "2025-02-06T00:00:00Z"
  }
]
```

---

#### `GET /api/portfolios/{id}`

Get a single portfolio item.

**Headers:** `Authorization: Bearer <token>`

**Response (200):** Portfolio object.
**Response (404):** `{ "error": "Portfolio item {id} not found" }`

---

#### `GET /api/portfolios/freelancer/{freelancer_id}`

Get all portfolio items for a specific freelancer.

**Headers:** `Authorization: Bearer <token>`

**Response (200):** Array of portfolio objects.

---

#### `POST /api/portfolios`

Create a portfolio item. The `freelancer_id` in the body must match the authenticated user.

**Headers:** `Authorization: Bearer <token>`

**Body:**

```json
{
  "title": "My Project",
  "description": "A cool project I built",
  "freelancer_id": "your-user-uuid",
  "price": 300.0
}
```

**Response (201):** Created portfolio object.
**Response (403):** `{ "error": "You can only create portfolio items for your own account" }`

---

#### `PUT /api/portfolios/{id}`

Update a portfolio item. Must be the owner.

**Headers:** `Authorization: Bearer <token>`

**Body:**

```json
{
  "title": "Updated title",
  "description": "Updated description",
  "price": 400.0
}
```

All fields are optional.

**Response (200):** Updated portfolio object.
**Response (403):** `{ "error": "You can only update your own portfolio items" }`

---

#### `DELETE /api/portfolios/{id}`

Delete a portfolio item. Must be the owner.

**Headers:** `Authorization: Bearer <token>`

**Response (200):** `{ "message": "Portfolio item {id} deleted" }`
**Response (403):** `{ "error": "You can only delete your own portfolio items" }`

---

### Contracts

Contracts represent a client's request to hire a freelancer for a specific gig. The flow is:

1. A **freelancer** posts a gig offering their services.
2. A **client** browses gigs and sends a contract request (`POST /api/contracts`).
3. The **freelancer** (gig owner) accepts or rejects the contract (`PUT /api/contracts/{id}/status`).

Only one contract per client per gig is allowed (enforced by a unique constraint).

#### `POST /api/contracts`

Create a contract request on a freelancer's gig. The `user_id` is automatically set from the authenticated user (the client).

**Headers:** `Authorization: Bearer <token>`

**Body:**

```json
{
  "gig_id": "uuid-of-the-gig"
}
```

**Response (201):** Created contract object.

```json
{
  "id": "uuid",
  "gig_id": "uuid",
  "user_id": "uuid",
  "status": "Pending",
  "created_at": "2025-02-10T00:00:00Z"
}
```

**Response (400):** `{ "error": "You cannot create a contract on your own gig" }`
**Response (404):** `{ "error": "Gig {id} not found" }`
**Response (409):** `{ "error": "You have already sent a contract request for this gig" }`

---

#### `GET /api/contracts`

List all contracts relevant to the authenticated user (where the user is either the client or the gig owner).

**Headers:** `Authorization: Bearer <token>`

**Response (200):** Array of contract objects.

---

#### `GET /api/contracts/{id}`

Get a single contract. Only the client or the gig owner can view it.

**Headers:** `Authorization: Bearer <token>`

**Response (200):** Contract object.
**Response (403):** `{ "error": "You can only view contracts you are involved in" }`
**Response (404):** `{ "error": "Contract {id} not found" }`

---

#### `PUT /api/contracts/{id}/status`

Accept or reject a contract. Only the gig owner (freelancer) can update the status, and only while the contract is Pending.

**Headers:** `Authorization: Bearer <token>`

**Body:**

```json
{
  "status": "Accepted"
}
```

Valid statuses: `"Accepted"`, `"Rejected"`.

**Response (200):** Updated contract object.
**Response (400):** `{ "error": "Contract is already Accepted. Only pending contracts can be updated." }`
**Response (403):** `{ "error": "Only the gig owner (freelancer) can accept or reject contracts" }`
**Response (404):** `{ "error": "Contract {id} not found" }`

---

#### `DELETE /api/contracts/{id}`

Withdraw a pending contract request. Only the client who created it can withdraw, and only while Pending.

**Headers:** `Authorization: Bearer <token>`

**Response (200):** `{ "message": "Contract {id} withdrawn" }`
**Response (400):** `{ "error": "Contract is already Accepted. Only pending contracts can be withdrawn." }`
**Response (403):** `{ "error": "You can only withdraw your own contract requests" }`
**Response (404):** `{ "error": "Contract {id} not found" }`

---

#### `GET /api/contracts/gig/{gig_id}`

Get all contracts for a specific gig. Only the gig owner (freelancer) can view these.

**Headers:** `Authorization: Bearer <token>`

**Response (200):** Array of contract objects.
**Response (403):** `{ "error": "Only the gig owner can view contracts for this gig" }`
**Response (404):** `{ "error": "Gig {gig_id} not found" }`

---

#### `GET /api/contracts/user/{user_id}`

Get all contracts sent by a specific user (as a client). Users can only view their own.

**Headers:** `Authorization: Bearer <token>`

**Response (200):** Array of contract objects.
**Response (403):** `{ "error": "You can only view your own contracts" }`

---

## Database Schema

### users

| Column        | Type         | Notes                           |
|---------------|--------------|----------------------------------|
| id            | UUID (PK)    | From Supabase auth               |
| email         | VARCHAR      | Unique                           |
| username      | VARCHAR      | Unique, nullable                 |
| display_name  | VARCHAR      | Nullable                         |
| avatar_url    | VARCHAR      | Nullable                         |
| auth_provider | VARCHAR      | e.g. "google"                    |
| role          | VARCHAR      | "client", "freelancer", "admin"  |
| created_at    | TIMESTAMPTZ  |                                  |
| updated_at    | TIMESTAMPTZ  | Nullable                         |

### gigs

| Column      | Type         | Notes                    |
|-------------|--------------|--------------------------|
| id          | UUID (PK)    |                          |
| title       | VARCHAR      |                          |
| description | TEXT         |                          |
| price       | DOUBLE       |                          |
| user_id     | UUID (FK)    | References users(id)     |
| created_at  | TIMESTAMPTZ  |                          |

### contracts

| Column     | Type         | Notes                    |
|------------|--------------|--------------------------|
| id         | UUID (PK)    |                          |
| gig_id     | UUID (FK)    | References gigs(id)      |
| user_id    | UUID (FK)    | References users(id)     |
| status     | VARCHAR      | "pending", "accepted", "rejected" |
| created_at | TIMESTAMPTZ  |                          |

**Constraints:** `UNIQUE(gig_id, user_id)` — one contract per client per gig.

### portfolios

| Column        | Type         | Notes                    |
|---------------|--------------|--------------------------|
| id            | UUID (PK)    |                          |
| title         | VARCHAR      |                          |
| description   | TEXT         |                          |
| freelancer_id | UUID (FK)    | References users(id)     |
| price         | DOUBLE       |                          |
| created_at    | TIMESTAMPTZ  |                          |

---

## Project Structure

```
gradwork-backend/
  src/
    main.rs              # Server entrypoint
    lib.rs               # Module exports
    auth/
      middleware.rs      # AuthenticatedUser extractor (JWT validation)
      jwks.rs            # JWKS cache for Supabase token verification
      jwt.rs             # JWT claims and validation
    handlers/
      mod.rs             # Route registration
      auth.rs            # /api/auth/* handlers
      users.rs           # /api/users/* handlers
      gigs.rs            # /api/gigs/* handlers
      portfolio.rs       # /api/portfolios/* handlers
      contracts.rs       # /api/contracts/* handlers
    db/
      mod.rs             # Database pool creation
      users.rs           # User DB queries
      gigs.rs            # Gig DB queries
      portfolio.rs       # Portfolio DB queries
      contracts.rs       # Contract DB queries
    models/
      mod.rs             # Module exports
      users.rs           # User entity + DTOs
      gigs.rs            # Gig entity + DTOs
      portfolio.rs       # Portfolio entity + DTOs
      contracts.rs       # Contract entity + DTOs
  migration/
    src/
      lib.rs             # Migration registry
      main.rs            # Migration CLI entrypoint
      m20250206_*.rs     # Migration files
      m20250207_*.rs
      m20250208_*.rs
```
