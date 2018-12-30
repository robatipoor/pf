# pf
[![Crates.io](https://img.shields.io/crates/v/pf.svg?style=plastic)](http://crates.io/crates/pf)
[![Build Status](https://travis-ci.org/robatipoor/pf.svg?branch=master)](https://travis-ci.org/robatipoor/pf)
[![Build status](https://ci.appveyor.com/api/projects/status/d2we8j2c58n6wq7o?svg=true)](https://ci.appveyor.com/project/robatipoor/pf)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

client tool for [paste.rs](https://paste.rs)


**install**

```sh
cargo install pf
```

**Build and install**

```sh
# build and install pf 
git clone https://github.com/robatipoor/pf \
&& cd pf \
&& make 
```

**Build dependency**

git, rustc, cargo, gnu make, binutils, upx

**run**

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
