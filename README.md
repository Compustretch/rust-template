# {{ .AppName }}

Generated via **Compustretch Devo IDP**.

## Running Locally

### Option A: Devo CLI (Recommended)

```bash
devo dev
```

### Option B: Native Rust & Docker

1. Setup environment configuration:
   ```bash
   cp .env.example .env
   ```
2. Start local platform capabilities:
   ```bash
   docker compose up -d
   ```
3. Run the service:
   ```bash
   cargo run
   ```
