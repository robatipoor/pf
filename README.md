# PasteFile 📁 📥

![License](https://img.shields.io/github/license/robatipoor/pf)
[![Lines Of Code](https://tokei.rs/b1/github/robatipoor/pf?category=code)](https://github.com/robatipoor/pf)
[![Format check](https://github.com/robatipoor/pf/actions/workflows/code-formater.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/code-formater.yml)
[![Build Check](https://github.com/robatipoor/pf/actions/workflows/build-checker.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/build-checker.yml)
[![Test](https://github.com/robatipoor/pf/actions/workflows/test.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/test.yml)
[![Clippy Check](https://github.com/robatipoor/pf/actions/workflows/code-linter.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/code-linter.yml)
[![Docker Image](https://github.com/robatipoor/pf/actions/workflows/image-builder.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/image-builder.yml)
[![Test Coverage](https://github.com/robatipoor/pf/actions/workflows/test-coverage.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/test-coverage.yml)
[![Codecov](https://codecov.io/gh/robatipoor/pf/branch/main/graph/badge.svg?token=BIMUKRJPE7)](https://codecov.io/gh/robatipoor/pf)

### Requirements

- [rust](https://www.rust-lang.org/tools/install)
- [docker](https://www.docker.com/)
- [docker-compose](https://docs.docker.com/compose/)

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
# Upload a file.
$ curl -F "file=@file.txt" 127.0.0.1:8080/upload
# Download file.
$ curl -X GET http://127.0.0.1:8080/{code}/{file_name}.{extension}
# Upload a file with basic authentication.
$ curl -u username:password -F "file=@file.txt" 127.0.0.1:8080/upload
# Download file with basic authentication.
$ curl -u username:password -X GET http://127.0.0.1:8080/{code}/{file_name}.{extension}
# Upload a file and then display the QR code.
$ curl -s -F "file=@file.txt" 127.0.0.1:8080/upload | jq -r '.qrcode' | base64 -d; echo
# Upload a file with an expiration time of 100 seconds.
$ curl -F "file=@file.txt" 127.0.0.1:8080/upload\?expire_secs=100
# Upload a file with a restriction on the number of downloads.
$ curl -F "file=@file.txt" 127.0.0.1:8080/upload\?max_download=10
# Upload a file with a restriction on the number of downloads.
$ curl -F "file=@file.txt" 127.0.0.1:8080/upload\?code_length=5
# Delete file.
$ curl -X DELETE http://127.0.0.1:8080/{code}/{file_name}.{extension}
# Get metadata file.
$ curl -X GET http://127.0.0.1:8080/{code}/{file_name}.{extension}/info
```


### Feature highlights

* Dependabot configuration

### Run tests

```
./test.sh
```
### Configuration

This project uses [config-rs](https://github.com/mehcode/config-rs) to manage configuration.
#### Configure with toml files
```bash
settings
├── base.toml # default config file 

```
#### Override configure with environment variables
```bash
export PF__SERVER__PORT=8080
export PF__SERVER__HOST=127.0.0.1
```
### Check code formatting and typo at commit time
```
cp ./scripts/git-hooks/* ./.git/hooks/
```
## License

Licensed under either of

 * MIT license
   ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)

## Contributing

Contributors are welcome, please fork and send pull requests! If you find a bug
or have any ideas on how to improve this project please submit an issue.

See [CONTRIBUTING.md](CONTRIBUTING.md).
