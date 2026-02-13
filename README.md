# drom
A task runner, written in Rust

# Setup

`docker build -t gemini-rust -f Dockerfile.rust .` - to build the Docker image

Add this to the settings:
```
  "hooksConfig": {
    "enabled": true
  }
```

Then install Conductor with: `gemini extensions install https://github.com/gemini-cli-extensions/conductor --auto-update`

`/conductor:setup` was run to configure the files in the `conductor` directory


