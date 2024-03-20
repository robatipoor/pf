# PasteFile 📁 📥
The service provides a convenient and efficient way to share files. The code includes the server API and CLI client tool, encompassing everything you need to create your own instance.

![License](https://img.shields.io/github/license/robatipoor/pf)
[![Lines Of Code](https://tokei.rs/b1/github/robatipoor/pf?category=code)](https://github.com/robatipoor/pf)
[![Format check](https://github.com/robatipoor/pf/actions/workflows/code-formater.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/code-formater.yml)
[![Build Check](https://github.com/robatipoor/pf/actions/workflows/build-checker.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/build-checker.yml)
[![Test](https://github.com/robatipoor/pf/actions/workflows/test.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/test.yml)
[![Clippy Check](https://github.com/robatipoor/pf/actions/workflows/code-linter.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/code-linter.yml)
[![Docker Image](https://github.com/robatipoor/pf/actions/workflows/image-builder.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/image-builder.yml)
[![Codecov](https://codecov.io/gh/robatipoor/pf/branch/main/graph/badge.svg?token=BIMUKRJPE7)](https://codecov.io/gh/robatipoor/pf)

**Feature highlights**

* Basic Authentication
* File Expiration
* Burn After Reading
* Large File Support
* QR code Generator
* Command Line Interface
* ChaCha20-Poly1305 Encryption
* Built-in TLS Server


**Run API Service Locally**

```sh
# Clone the project
$ git clone https://github.com/robatipoor/pf

# Build the project
$ cargo build --bin pf-api --release

# Run the backend on address 127.0.0.1:8080
$ ./target/release/pf-api --settings api/settings/base.toml

# Alternatively, Run backend with cargo
$ cargo run --bin pf-api
```
**Run API Service via Docker**

```sh
# Pull the image from the github registry and run container
$ docker run --name pf-api --rm -p 8080:8080 -d ghcr.io/robatipoor/pf-api:latest

# Alternatively, build your own Docker image.
$ docker build -t pf-api:latest -f api/Dockerfile .

# Run Docker container on address 0.0.0.0:8080
$ docker run --name pf-api --rm -p 8080:8080 -d pf-api:latest
```

**How to Use**

```sh
# Open it in the default browser.
$ open http://127.0.0.1:8080

# Ping the server.
$ curl -X GET http://127.0.0.1:8080/healthz

# Upload a file and retrieve the corresponding download URL.
$ curl -s -F "file=@{file_name}" 127.0.0.1:8080/upload | jq -r '.url'

# Download a file.
$ curl -o {file_name} http://127.0.0.1:8080/{code}/{file_name}

# Upload a file with basic authentication.
$ curl -u username:password -F "file=@{file_name}" 127.0.0.1:8080/upload

# Download a file with basic authentication.
$ curl -o {file_name} -u username:password http://127.0.0.1:8080/{code}/{file_name}

# Upload a file and then display the QR code.
$ curl -s -F "file=@{file_name}" 127.0.0.1:8080/upload\?qr_code_format=text \
| jq -r '.qr_code' | base64 -d; echo

# Upload a file with an expiration time of 100 seconds (default value specified in settings file).
$ curl -F "file=@{file_name}" 127.0.0.1:8080/upload\?expire_secs=100

# Upload a file with a restriction on the number of downloads.
$ curl -F "file=@{file_name}" 127.0.0.1:8080/upload\?max_download=10

# Upload a file and specify the minimum code length in the URL path as 5 (default value specified in settings file).
$ curl -F "file=@{file_name}" 127.0.0.1:8080/upload\?code_length=5

# Upload a file and prevent manual deletion until expiration.
$ curl -F "file=@{file_name}" 127.0.0.1:8080/upload\?allow_manual_deletion=false

# Get metadata for a file.
$ curl -X GET http://127.0.0.1:8080/info/{code}/{file_name}

# Delete a file.
$ curl -X DELETE http://127.0.0.1:8080/{code}/{file_name}
```

**API settings**

```toml
# api/settings/base.toml
# Maximum upload size in bytes
max_upload_bytes_size = 1_000_000_000 # 1GB

# Default code length in the url path
default_code_length = 3

# Default expiration time in seconds
default_expire_secs = 7200

# Allow manual deletion of files.
allow_manual_deletion = true

# Server configuration section
[server]
# Communication protocol (e.g., "http" or "https")
schema = "http"

# Host IP address for the server
host = "127.0.0.1"

# Port number for the server
port = 8080

# Domain name URL
domain_name = "localhost:8080"

# Public IP address
public_addr = "192.168.1.1:8080"

# TLS key file path
file_tls_key_path = "key.pem"

# TLS certificate file path
file_tls_cert_path = "cert.pem"

# File system configuration section
[fs]
# Base directory for file system operations
base_dir = "tmp/fs"

# Database configuration section
[db]
# Path directory to the database file
path_dir = "tmp/db"
```

**Override settings with environment variables**

```sh
export PF__SERVER__PORT=8080
export PF__SERVER__HOST=127.0.0.1
```

**PastFile Command Line Interface**

```sh
# Install CLI tool
$ cargo install --path cli

# Define an alias in the shell profile for 'pf' with server address
$ alias pf="pf-cli --server-addr http://localhost:8080"

# Copy text content to the server with an expiration time of 10 minutes.
$ echo 'Hello World!' | pf copy --expire "10 minute"

# Paste text content into the file.
$ pf paste --url-path "{code}/{file_name}" > my_file.txt

# Encrypt and upload a file and retrieve the corresponding download URL.
$ pf upload --source-file ~/example-file.txt --key-nonce "{key}:{nonce}" --progress-bar

# Download and decrypt a file.
$ pf download --destination ~/example-dir/ --url-path "{code}/{file_name}" --key-nonce "{key}:{nonce}"

# Upload a file and then display the QR code.
$ pf upload --source-file ~/example-file.txt --output qr-code

# Get metadata for a file.
$ pf info --url-path "{code}/{file_name}"

# Delete a file.
$ pf delete --url-path "{code}/{file_name}"

```

**Run tests**
```sh
# Execute all test projects.
$ ./test
```

**Check code formatting and typo at commit time**

```sh
$ cp ./scripts/git-hooks/* ./.git/hooks/
```

**License**

Licensed under either of

 * MIT license
   ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)

**Contributing**

Contributors are welcome, please fork and send pull requests! If you find a bug
or have any ideas on how to improve this project please submit an issue.

See [CONTRIBUTING.md](CONTRIBUTING.md).
