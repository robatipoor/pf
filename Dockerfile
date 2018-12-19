FROM ubuntu:latest
ADD target/release/pf /
CMD ["/pf"]
