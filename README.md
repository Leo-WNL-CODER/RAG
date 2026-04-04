# RAG Server Backend

This is the backend architecture for the RAG (Retrieval-Augmented Generation) application, built primarily with asynchronous Rust (`axum`) and a lightweight Python microservice for document ingestion.

## Architecture Overview

The system is highly concurrent and designed to fit within free-tier cloud environments (like Render, Fly.io, etc.). It consists of the following components:

### 1. **Rust API Server (`axum`)**
- Acts as the primary entry point for all frontend requests.
- Handles user authentication (JWT + HTTP-only Cookies).
- Orchestrates document parsing, chunking, embedding generation, and vector storage.
- Interacts with the LLM for query answering.

### 2. **Document Parser Service (Python / `FastAPI`)**
- A lightweight secondary microservice used strictly for extracting raw text from uploaded files.
- Uses `pypdf` for PDFs, `python-docx` for Word documents, and pure decoding for text files.
- Avoiding heavy libraries (like LibreOffice or full `unstructured`) ensures the service runs comfortably in <512MB RAM without OOM crashes.

### 3. **Embedding Model (`ort` + Quantized ONNX)**
- Embeddings are generated **locally** within the Rust server process using the `ort` crate.
- Uses a quantized version of the **BAAI/bge-base-en-v1.5** model (~110MB). 
- Using a quantized model avoids out-of-memory errors on free-tier containers while retaining excellent embedding accuracy.

### 4. **LLM Provider**
- Uses **Groq API** as the LLM provider.
- Groq provides ultra-fast inference and a highly compatible API, substituting heavier local models like Ollama for production deployment.

### 5. **Databases**
- **Relational DB (PostgreSQL / Neon)**: Stores users, hashed passwords (argon2), document metadata (`doc_info`), and conversation summaries.
- **Vector DB (Qdrant)**: Stores document embeddings (chunked text vectors) and associates them with unique `user_id`s and `doc_id`s, enabling vector similarity search during Q&A.
- **Redis (Upstash)**: Used for session/state management if needed, and as a fast ephemeral cache.

---

## Local Development Setup

To run this project locally, you need both the Rust server and the Python parser running simultaneously.

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (Ensure you are on the latest stable version, >= 1.88 recommended)
- [Python 3.11+](https://www.python.org/downloads/)
- Existing cloud databases (Neon Postgres, Qdrant Cloud, Upstash Redis), or local Docker equivalents.
- Optional: `sqlx-cli` if you need to run migrations (`cargo install sqlx-cli`).

### 1. Environment Variables
Create a `.env` file in the root `RAG_Server` directory matching the following structure:
```env
DATABASE_URL="postgresql://user:pass@host/dbname"
JWT_ACCESS_TOKEN_SECRET="your-access-secret"
JWT_REFRESH_TOKEN_SECRET="your-refresh-secret"
REDIS_URL="rediss://default:pass@host:port"
GROQ_API_KEY="gsk_yourkey"
QDRANT_API_KEY="your-qdrant-key"
QDRANT_URL="https://your-qdrant-cluster-url:6334"
PYTHON_PARSER_URL="http://127.0.0.1:8000/parse"
FRONTEND_URL="http://localhost:5173"
PORT="3001"
```

### 2. Start the Python Parser
The Python service uses lightweight dependencies to extract text from files.
```bash
cd python_parser_service
# Create and activate a virtual environment
python -m venv venv
source venv/bin/activate  # Or `venv\Scripts\activate` on Windows

# Install dependencies
pip install fastapi uvicorn python-multipart pypdf python-docx python-magic

# Start the parser
uvicorn main:app --host 127.0.0.1 --port 8000
```

### 3. Start the Rust Server
In a new terminal, navigate to the `RAG_Server` root. Note that on its first run, `ort` will download the ONNX Runtime C/C++ binaries automatically. You will need the embedding model in the `models/` directory for local development (which Render does during Docker build).

```bash
cd RAG_Server

# Download the model manually if not present
mkdir -p models
curl -L "https://huggingface.co/Xenova/bge-base-en-v1.5/resolve/main/onnx/model_quantized.onnx" -o models/model.onnx

# Build and run the server
cargo run
```

The Rust server will default to `http://0.0.0.0:3001` (or whatever `PORT` is defined in `.env`).

---

## Deployment Notes
- **Cookies & CORS**: The server is configured for cross-origin authentication with the frontend. Cookies have `Secure=true`, `SameSite=None`. Because of this, local development ideally requires the frontend to run on HTTPS (e.g., using `vite-plugin-mkcert`) or adjusting cookie settings for standard HTTP localhost.
- **Model Distribution**: The 415MB full precision ONNX model was dropped in favor of the quantized version. Instead of tracking the model in Git LFS, the Render `Dockerfile` automatically downloads the `~110MB` model from HuggingFace at build time into `/app/models/model.onnx`.
- **Cold Starts**: If utilizing a free tier for the Python parser, note that a 'cold start' may take ~30-60s to wake up the service. The Rust `reqwest` client has a custom 90-second timeout configured to gracefully handle this.
