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
$ docker run --name pf-api --rm -p 8080:8080 -e PF__SERVER__HOST='0.0.0.0' -d pf-api:latest
# Alternatively, you can pull the image from the github registry
$ docker pull ghcr.io/robatipoor/pf-api:latest
# Run Docker container on address 0.0.0.0:8080
$ docker run --name pf-api --rm -p 8080:8080 -e PF__SERVER__HOST='0.0.0.0' -d ghcr.io/robatipoor/pf-api:latest
```

**How to Use**

```sh
# Ping the server
$ curl -X GET http://127.0.0.1:8081/healthz
# Upload file
$ curl -F "file=@file.txt" 127.0.0.1:8080/upload\?expire_secs=100
$ curl -F "file=@file.txt" 127.0.0.1:8080/upload\?delete_manually=true
$ curl -F "file=@file.txt" 127.0.0.1:8080/upload\?code_length=10
$ curl -F "file=@file.txt" 127.0.0.1:8080/upload\?max_download=5
```


### Feature highlights

* Dependabot configuration

### Run tests
```
./test.sh
```
![api grid](https://codecov.io/gh/robatipoor/pf/branch/main/graphs/tree.svg?token=BIMUKRJPE7)
### Configuration
This project uses [config-rs](https://github.com/mehcode/config-rs) to manage configuration.
#### Configure with toml files
```bash
settings
├── base.toml # default config file 

```
#### Configure with environment variables
```bash
export APP_SERVER__PORT=8080
export APP_SERVER__HOST=127.0.0.1
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
