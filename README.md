# pf
[![Crates.io](https://img.shields.io/crates/v/pf.svg?style=plastic)](http://crates.io/crates/pf)
![License](https://img.shields.io/github/license/robatipoor/pf)
![Lines of code](https://img.shields.io/tokei/lines/github/robatipoor/pf)
[![Format check](https://github.com/robatipoor/pf/actions/workflows/format.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/format.yml)
[![Build Check](https://github.com/robatipoor/pf/actions/workflows/check.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/check.yml)
[![Test](https://github.com/robatipoor/pf/actions/workflows/test.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/test.yml)
[![Clippy Check](https://github.com/robatipoor/pf/actions/workflows/clippy.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/clippy.yml)
[![Docker Image](https://github.com/robatipoor/pf/actions/workflows/build.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/build.yml)
[![Test Coverage](https://github.com/robatipoor/pf/actions/workflows/coverage.yml/badge.svg)](https://github.com/robatipoor/pf/actions/workflows/coverage.yml)
[![Codecov](https://codecov.io/gh/robatipoor/pf/branch/main/graph/badge.svg?token=BIMUKRJPE7)](https://codecov.io/gh/robatipoor/pf)
[![Dependency status](https://deps.rs/repo/github/robatipoor/pf/status.svg)](https://deps.rs/repo/github/robatipoor/pf)


**install**

```sh
cargo install pf
```

**Build and install**

```sh
# dependencies git, rustc, cargo, gnu make, binutils, upx
# build and install pf 
git clone https://github.com/robatipoor/pf \
&& cd pf \
&& make 
```


**how to use command**

```sh
# post string
echo 'Hello !' | pf
# post file
pf some-file.txt
# get file 
pf https://paste.rs/some
# delete file
pf -d https://paste.rs/some
# read log file
pf --log
```

**how to use crate**
```rust
extern crate pf;

use pf::PastFile;

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
