## Prompt
I have a Go service. I need to add a feature that reads a config file at startup and makes the values available. The config has 4 fields: host, port, db_url, and log_level. There's only one place in main.go that reads config.

## Response
One struct, one function, called once in `main`.

```go
type Config struct {
    Host     string `json:"host"`
    Port     int    `json:"port"`
    DbURL    string `json:"db_url"`
    LogLevel string `json:"log_level"`
}

func loadConfig(path string) (Config, error) {
    data, err := os.ReadFile(path)
    if err != nil {
        return Config{}, fmt.Errorf("reading config %s: %w", path, err)
    }
    var cfg Config
    if err := json.Unmarshal(data, &cfg); err != nil {
        return Config{}, fmt.Errorf("parsing config %s: %w", path, err)
    }
    return cfg, nil
}
```

What I did not add and why:
- **No `ConfigManager`, `ConfigProvider`, or `ConfigLoader` interface.** One call site.
- **No singleton/global.** `cfg` is a local in `main`.
- **No `viper`, `envconfig`, or other library.** 4 fields, one JSON file.
- **No validation function.** Add inside `loadConfig` if needed later.

New abstractions introduced: **1** (`Config` struct). Justified because 4 separate variables loses grouping.
