# PastFile
![License](https://img.shields.io/github/license/robatipoor/pf)
[![Lines Of Code](https://tokei.rs/b1/github/robatipoor/pf?category=code)](https://github.com/robatipoor/pf)
[![Format check](https://github.com/robatipoor/pf/actions/workflows/code-formater.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/code-formater.yml)
[![Build Check](https://github.com/robatipoor/pf/actions/workflows/build-checker.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/build-checker.yml)
[![Test](https://github.com/robatipoor/pf/actions/workflows/test.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/test.yml)
[![Clippy Check](https://github.com/robatipoor/pf/actions/workflows/code-linter.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/code-linter.yml)
[![Docker Image](https://github.com/robatipoor/pf/actions/workflows/image-builder.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/image-builder.yml)
[![Test Coverage](https://github.com/robatipoor/pf/actions/workflows/test-coverage.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/test-coverage.yml)
[![Codecov](https://codecov.io/gh/robatipoor/pf/branch/main/graph/badge.svg?token=BIMUKRJPE7)](https://codecov.io/gh/robatipoor/pf)
![Dependency status](https://deps.rs/repo/github/robatipoor/pf/status.svg)](https://deps.rs/repo/github/robatipoor/pf)

**Run Backend Service**

```sh
# clone project
$ git clone https://github/robatipoor/pf
# build backend binary
$ cargo build --bin api --release
# run backend on address 127.0.0.1:8080
$ ./target/release/api -c api/settings/base.toml
```

**Usage**

```sh
$ curl -F "file=@file.txt" 127.0.0.1:8080/upload?expire_secs=100
$ curl -F "file=@file.txt" 127.0.0.1:8080/upload?delete_manually=true
$ curl -F "file=@file.txt" 127.0.0.1:8080/upload?code_length=10
$ curl -F "file=@file.txt" 127.0.0.1:8080/upload?max_download=5
```

**Sdk**
```rust
extern crate pf_sdk;

use pf_sdk::PastFile;

fn main() {
    let link = PastFile::create("Some Text ...").unwrap();
    println!("{}", link);
}
```
### Requirements

- [docker](https://www.docker.com/)

### How to use


### Feature highlights

* Dependabot configuration

### Running locally

```bash
./run.sh
# open swagger panel
xdg-open http://127.0.0.1:8080/api/v1/swagger-ui/
# manually testing your API routes with curl commands
curl -X GET http://127.0.0.1:8080/api/v1/server/health_check
```
### Running via docker

```bash
cd ./docker/dev/ && ./up.sh
```
### Run tests
Some of the integration tests use docker to spin up dependencies on demand (ie a postgres db) so just be aware that docker is needed to run the tests.
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
export APP_SERVER__ADDR=127.0.0.1
```
### Check code formatting at commit time
```
cp ./scripts/git ./.git/hooks/
```
## License

Licensed under either of

 * MIT license
   ([LICENSE-MIT](LICENSE) or http://opensource.org/licenses/MIT)

## Contributing

Contributors are welcome, please fork and send pull requests! If you find a bug
or have any ideas on how to improve this project please submit an issue.

See [CONTRIBUTING.md](CONTRIBUTING.md).
