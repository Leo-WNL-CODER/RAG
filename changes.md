# RAG Server — Production Readiness Changes

## 1. Critical Security Issues

### Hardcoded/Weak Secrets
- JWT secrets are `"ACCESS"` and `"REFRESH"` — use 256-bit random strings
- Redis URL is hardcoded as a fallback in `src/db/start_redis.rs` — remove it
- `.env` contains real database credentials — ensure `.gitignore` covers it before pushing

### Frontend Password Fields
- `SingIn.jsx` and `Signup.jsx` use `type="text"` — change to `type="password"`

### No Rate Limiting
- Add `tower::limit::RateLimitLayer` or equivalent middleware to prevent brute-force attacks on `/signin`

### No CSRF Protection
- Add `SameSite=Strict` or `SameSite=Lax` to cookie settings

### Access Token on Refresh Too Short
- `src/routes/refetch_access_token.rs` sets 15 seconds — bump to 15 minutes

---

## 2. Error Handling & Reliability

- Replace all `.unwrap()` calls with proper error handling (especially in `parse_dox.rs`, `querying.rs`, `ask_model.rs`)
- Add structured logging — use `tracing` + `tracing-subscriber` instead of `println!`
- Add health check endpoint — `GET /health` that checks DB, Qdrant, Redis, and Ollama connectivity
- Handle Python parser failures gracefully — return a clear error instead of panicking

---

## 3. Architecture Improvements

| Issue | Fix |
|---|---|
| `Mutex<Session>` for ONNX blocks async threads | Use `tokio::sync::Mutex` or run inference in `spawn_blocking` |
| Single active document per user | Allow multiple documents, add a `DELETE /doc/:id` endpoint |
| No request validation | Add payload size limits, query length limits |
| Qdrant dependency on git master branch | Pin to a released version |
| No graceful shutdown | Add `tokio::signal` handling for clean shutdown |
| CORS allows only localhost | Make CORS origins configurable via env var |
| No API versioning | Prefix routes with `/api/v1/` |

---

## 4. Code Quality

- **Typo**: `middlwares/` → `middlewares/`
- **Typo**: `SingIn.jsx` → `SignIn.jsx`
- Remove `rag_env/` and `qdrant_storage/` from the repo (add to `.gitignore`)
- Add `Dockerfile` for the Rust backend and Python service
- Add `.dockerignore` to exclude `models/`, `node_modules/`, `target/`
- Pin dependency versions — avoid `*` or git `master` branch deps

---

## 5. Performance

- Connection pooling: Max 5 PostgreSQL connections is fine for free tier, but make it configurable
- Qdrant search limit: 8 results is reasonable, but make it configurable
- Model loading: 435MB ONNX model loads at startup — add a startup readiness probe
- Frontend: Build the React app and serve it from the Rust server (eliminates CORS issues and simplifies deployment)

---

## 6. Missing Features for Production

- Request logging/monitoring (use `tower-http::trace`)
- API documentation (consider `utoipa` for OpenAPI/Swagger)
- Database migrations on startup (run `sqlx migrate run` automatically)
- Environment-based config (dev/staging/prod profiles)
- Tests — no tests exist; add at least integration tests for auth and query flows

---

## 7. Free Deployment Strategy

### Recommended Free Architecture

```
Vercel (Frontend)
    |
Fly.io or Render (Rust Backend)
    |-- Neon (PostgreSQL) — free
    |-- Qdrant Cloud (vectors) — free
    |-- Upstash Redis — free
    |-- Groq API (LLM) — free
```

### Service-by-Service

| Service | Platform | Free Tier Details |
|---|---|---|
| **Rust Backend** | Fly.io or Render.com | Fly: 3 shared VMs, 256MB RAM. Render: 512MB RAM |
| **Vector DB** | Qdrant Cloud | 1GB storage, 1 cluster free |
| **PostgreSQL** | Neon (already using) | 0.5GB storage, serverless, auto-suspend |
| **Redis** | Upstash (already using) | 10,000 commands/day free |
| **LLM** | Groq / Google AI Studio / Cloudflare Workers AI | Groq: free Llama 3/Mistral. Gemini: generous free limits |
| **Frontend** | Vercel or Cloudflare Pages | Generous free tiers for static sites |

### Eliminate Python Parser Service
- Only 22 lines, calls `unstructured.partition()`
- Replace with Rust crates: `pdf-extract`/`lopdf` for PDFs, `docx-rs` for DOCX
- Removes an entire service to deploy

### Replace Ollama with Free LLM API
- Ollama requires GPU/strong CPU — not free-tier friendly
- Groq API is OpenAI-compatible, very fast, and free
- Change `ask_model.rs` to call Groq's API instead

### Deployment Steps

1. Dockerize the backend (sample Dockerfile below)
2. Deploy to Fly.io: `fly launch` then `fly deploy`
3. Deploy frontend to Vercel: `vercel --prod` from `RagFrontend/`
4. Set up Qdrant Cloud: create free cluster, update connection URL
5. Replace Ollama with Groq: change `ask_model.rs` to call Groq API

### Sample Dockerfile

```dockerfile
FROM rust:1.82-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rag_server /usr/local/bin/
COPY --from=builder /app/models/ /app/models/
CMD ["rag_server"]
```

### ONNX Model Note
- The 435MB model won't fit in 256MB RAM (Fly.io free tier)
- Options: use a smaller model like `all-MiniLM-L6-v2` (~80MB, 384 dims) or use Render (512MB RAM)

---

## 8. Priority Summary

| Priority | Change |
|---|---|
| **P0** | Fix JWT secrets, remove hardcoded Redis URL, fix password input types |
| **P0** | Replace all `.unwrap()` with proper error handling |
| **P1** | Add `tracing` logging, health check endpoint |
| **P1** | Replace Ollama with Groq/Gemini API for deployability |
| **P1** | Eliminate Python parser service (use Rust crates) |
| **P2** | Add rate limiting, request validation |
| **P2** | Serve frontend from backend (or deploy separately) |
| **P2** | Add basic integration tests |
| **P3** | API versioning, OpenAPI docs, graceful shutdown |
