# PasteFile 📁 📥
The service provides a convenient means of sharing files without the necessity for user accounts or a complicated setup. The accompanying code includes the server, encompassing everything you need to create your own instance.

![License](https://img.shields.io/github/license/robatipoor/pf)
[![Lines Of Code](https://tokei.rs/b1/github/robatipoor/pf?category=code)](https://github.com/robatipoor/pf)
[![Format check](https://github.com/robatipoor/pf/actions/workflows/code-formater.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/code-formater.yml)
[![Build Check](https://github.com/robatipoor/pf/actions/workflows/build-checker.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/build-checker.yml)
[![Test](https://github.com/robatipoor/pf/actions/workflows/test.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/test.yml)
[![Clippy Check](https://github.com/robatipoor/pf/actions/workflows/code-linter.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/code-linter.yml)
[![Docker Image](https://github.com/robatipoor/pf/actions/workflows/image-builder.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/image-builder.yml)
[![Test Coverage](https://github.com/robatipoor/pf/actions/workflows/test-coverage.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/test-coverage.yml)
[![Codecov](https://codecov.io/gh/robatipoor/pf/branch/main/graph/badge.svg?token=BIMUKRJPE7)](https://codecov.io/gh/robatipoor/pf)

**Feature highlights**

* Basic Authentication
* Anonymous Uploads
* File Expiration
* Burn After Reading
* QR code Generator
* Command Line Interface (CLI)

**Run Backend Service Locally**

```sh
# Clone the project
$ git clone https://github.com/robatipoor/pf
# Build the backend binary
$ cargo build --bin api --release
# Run the backend on address 127.0.0.1:8080
$ ./target/release/api --settings api/settings/base.toml
```
**Run Backend Service via Docker**

```sh
# Build docker image
$ docker build -t pf-api:latest -f api/Dockerfile .
# Run Docker container on address 0.0.0.0:8080
$ docker run --name pf-api --rm -p 8080:8080 \
-e PF__SERVER__HOST='0.0.0.0' -d pf-api:latest
# Alternatively, you can pull the image from the github registry and run container
$ docker run --name pf-api --rm -p 8080:8080 \
-e PF__SERVER__HOST='0.0.0.0' -d ghcr.io/robatipoor/pf-api:latest
```

**How to Use**

```sh
# Ping the server.
$ curl -X GET http://127.0.0.1:8080/healthz
# Upload a file and then get download url.
$ curl -s -F "file=@{file_name}" 127.0.0.1:8080/upload | jq -r '.url'
# Download file.
$ curl -o {file_name} http://127.0.0.1:8080/{code}/{file_name}
# Upload a file with basic authentication.
$ curl -u username:password -F "file=@{file_name}" 127.0.0.1:8080/upload
# Download file with basic authentication.
$ curl -o {file_name} -u username:password http://127.0.0.1:8080/{code}/{file_name}
# Upload a file and then display the QR code.
$ curl -s -F "file=@{file_name}" 127.0.0.1:8080/upload | jq -r '.qrcode' | base64 -d; echo
# Upload a file with an expiration time of 100 seconds (default value specify in settings file).
$ curl -F "file=@{file_name}" 127.0.0.1:8080/upload\?expire_secs=100
# Upload a file with a restriction on the number of downloads.
$ curl -F "file=@{file_name}" 127.0.0.1:8080/upload\?max_download=10
# Upload a file and specify the minimum code length in the URL path as 5 (default value specify in settings file).
$ curl -F "file=@{file_name}" 127.0.0.1:8080/upload\?code_length=5
# Upload a file and prevent manual deletion until expiration.
$ curl -F "file=@{file_name}" 127.0.0.1:8080/upload\?delete_manually=false
# Get metadata file.
$ curl -X GET http://127.0.0.1:8080/info/{code}/{file_name}
# Delete file.
$ curl -X DELETE http://127.0.0.1:8080/{code}/{file_name}
```

**Backend settings**

```toml
# api/settings/base.toml
# Maximum upload size in megabytes
max_upload_size = 1024

# Default code length in the url
default_code_length = 3

# Default expiration time in seconds
default_expire_secs = 7200

# Server configuration section
[server]
# Communication protocol (e.g., "http" or "https")
schema = "http"

# Host IP address for the server
host = "127.0.0.1"

# Port number for the server
port = 8080

# File system configuration section
[fs]
# Base directory for file system operations
base_dir = "fs-tmp"

# Database configuration section
[db]
# Path to the database file
path = "db-tmp"
```

**Override settings with environment variables**

```bash
export PF__SERVER__PORT=8080
export PF__SERVER__HOST=127.0.0.1
```

**PastFile Command Line Interface**

```sh
# Clone the project
$ git clone https://github.com/robatipoor/pf
# Build the cli tool
$ cargo build --bin cli --release
# Upload a file and then get download url.
$ ./target/release/cli --server-addr "http://localhost:8080" \
upload --source-file ~/example-file.txt  
# Upload a file with basic authentication.
$ ./target/release/cli --server-addr "http://localhost:8080" \
--auth "username:password" upload --source-file ~/example-file.txt 
# Upload a file with an expiration time of 10 minutes.
$ ./target/release/cli --server-addr "http://localhost:8080" \
upload --expire "10 minute" --source-file ~/example-file.txt  
# Upload a file and then display the QR code.
$ ./target/release/cli --server-addr "http://localhost:8080" \
upload --source-file ~/example-file.txt --qrcode
# Download file.
$ ./target/release/cli --server-addr "http://localhost:8080" \
download --destination-dir ~/example-dir/
# Download file.
$ ./target/release/cli --server-addr "http://localhost:8080" \
download --destination-dir ~/example-dir/ --url-path "{code}/{file_name}"
# Get metadata file.
$ ./target/release/cli --server-addr "http://localhost:8080" \
info --url-path "{code}/{file_name}"
# Delete file.
$ ./target/release/cli --server-addr "http://localhost:8080" \
delete --url-path "{code}/{file_name}"
```

**Run tests**

```sh
./test.sh
```

**Check code formatting and typo at commit time**

```sh
cp ./scripts/git-hooks/* ./.git/hooks/
```

**License**

Licensed under either of

 * MIT license
   ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)

**Contributing**

Contributors are welcome, please fork and send pull requests! If you find a bug
or have any ideas on how to improve this project please submit an issue.

See [CONTRIBUTING.md](CONTRIBUTING.md).
