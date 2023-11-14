FROM debian:bullseye
# generate a ubuntu docker image install a glibc_2.38

WORKDIR /app

RUN apt-get update
RUN apt-get install glibc-doc libc6-dev -y