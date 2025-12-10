# Factorio Server Browser

A web-based Factorio multiplayer server browser with filtering, search, and player history (24h) tracking. Written almost entirely in [Rust](https://rust-lang.org/).

## Technology Stack

- **[Rust](https://rust-lang.org)** 
- **[Yew](https://yew.rs)** Server-side Rendering
- **[Surreal](https://surrealdb.com/)** Database
- **[TailwindCSS](https://tailwindcss.com/)** CSS Styling

## Features

- **Full Server list** from the official [Factorio Matchmaking API](https://wiki.factorio.com/Matchmaking_API)
- **Advanced filtering** by search title, description, tags, game version, player count, password protection, and dedicated server status.
- **Server detail pages** with:
  - Current online players
  - Complete mod list
  - 24-hour player count history chart
- Data refreshes automatically every minute

# Prerequisites

- [Rust](https://rust-lang.org/tools/install/) 1.91.1+
- [Tailwindcss](https://tailwindcss.com/) binary in your $PATH
- [Make](https://www.gnu.org/software/make/)
- Factorio account [token](https://www.factorio.com/profile) (for API access)
- SurrealDB instance (optional in dev, since it defaults to in-memory for development)

## Configuration

Create a `.env` file in the project root with the following variables:

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `FACTORIO_USERNAME` | Yes | — | Your Factorio.com username |
| `FACTORIO_TOKEN` | Yes | — | Your Factorio.com API token |
| `SURREAL_URL` | No | `mem://` | SurrealDB connection URL |
| `SURREAL_NS` | No | `factorio` | Database namespace |
| `SURREAL_DB` | No | `tracker` | Database name |
| `SURREAL_USER` | No | — | Database username |
| `SURREAL_PASS` | No | — | Database password |

### Obtaining Your Factorio API Token

0. Buy [Factorio](https://factorio.com)
1. Log in to [factorio.com](https://factorio.com)
2. Navigate to your [profile page](https://www.factorio.com/profile)
3. Copy your token to .env

## Local Development

1. **Clone the repository**
   ```bash
   git clone https://github.com/Psaltor/factory-browser.git
   cd factory-browser
   ```

2. **Create your environment file**
   ```bash
   cp .env.example .env
   # Edit .env with your Factorio credentials
   ```

3. **Run the development server**
   ```bash
   make dev
   ```

4. **Access the application** at [http://localhost:8000](http://localhost:8000)

## License

GPLv2 — see [LICENSE](LICENSE) for details.
