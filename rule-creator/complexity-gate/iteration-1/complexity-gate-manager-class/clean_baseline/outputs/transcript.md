Empty project. Here's exactly what I'd produce — two files:

**config.go** — the config struct and loader:

```go
package main

import (
	"encoding/json"
	"fmt"
	"os"
)

type Config struct {
	Host     string `json:"host"`
	Port     int    `json:"port"`
	DbURL    string `json:"db_url"`
	LogLevel string `json:"log_level"`
}

func LoadConfig(path string) (Config, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return Config{}, fmt.Errorf("reading config: %w", err)
	}
	var cfg Config
	if err := json.Unmarshal(data, &cfg); err != nil {
		return Config{}, fmt.Errorf("parsing config: %w", err)
	}
	return cfg, nil
}
```

**main.go** — reads config at startup:

```go
package main

import (
	"log"
	"os"
)

func main() {
	path := "config.json"
	if p := os.Getenv("CONFIG_PATH"); p != "" {
		path = p
	}

	cfg, err := LoadConfig(path)
	if err != nil {
		log.Fatal(err)
	}

	log.Printf("starting server on %s:%d (log_level=%s)", cfg.Host, cfg.Port, cfg.LogLevel)
	// use cfg.DbURL, cfg.Host, cfg.Port, cfg.LogLevel as needed
}
```

**config.json** example:

```json
{
  "host": "0.0.0.0",
  "port": 8080,
  "db_url": "postgres://localhost:5432/mydb",
  "log_level": "info"
}
```

Key decisions:

- **JSON** — stdlib only, no external dependencies. If you'd prefer TOML/YAML, let me know.
- **`LoadConfig` returns `(Config, error)`** — no globals, no `init()`. The caller owns the value.
- **`CONFIG_PATH` env var** — overrides the default path without flags.
- **Errors wrap with context** via `%w` so callers can inspect the cause.

Want me to write these files into the project?
