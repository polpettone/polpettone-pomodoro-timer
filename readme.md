# Pomodoro Timer

A command line pomodoro timer.

## Install

### Option 1
- clone repo and run cargo install --path .

### Option 2 
- prerequisite: archlinux/manjaro and yay

```
yay -S polpettone-pomodoro-timer
```

### Help 
For more commands run 

```
polpettone-pomodoro-timer help
```

## Development with Postgres

To run the application with a PostgreSQL database locally for end-to-end testing:

1. **Start Postgres via Docker Compose:**
   ```bash
   docker-compose up -d
   ```
   This starts a Postgres instance on `localhost:5432`.

2. **Configure the application:**
   You can either use a `.env` file or update your `config.toml`.
   
   Using `.env`:
   ```bash
   cp .env.example .env
   ```
   
   Or update `config.toml`:
   ```toml
   [pomodoro_config]
   persistence_mode = "postgres"
   database_url = "postgres://user:password@localhost:5432/polpettone"
   ```

3. **Initialize the Database:**
   The application automatically creates the necessary tables on startup if they don't exist. You can trigger this by running:
   ```bash
   cargo run -- init-session-dir
   ```

4. **Run the Backend Server:**
   ```bash
   cargo run -- server
   ```
