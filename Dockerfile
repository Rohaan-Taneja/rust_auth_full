# try slim also 
FROM rust:1.93   

# /adding system files which are non rust and rust crates depends on them 
# debian is a linux os , and its package mnager is apt-get, 
# so whatever we have to add on our linux os , we will istall that to by using apt-get
# THE APT-get manager first genrate a list of info of packages and save it and then it installs them
# update get the packages details like Package name
# Version , Download URL , Dependencies   => in /var/lib/apt/lists folder
# and install read those details and install them 
# it temporarily install .deb files in /var/cashe/apt/archives
# then in   /usr/lib , /usr/include , /usr/bin
# now after install package list is useless , so we will remove it after download , 
RUN apt-get update && apt-get install -y \
    libpq-dev \
    pkg-config \
    build-essential \
    && rm -rf /var/lib/apt/lists/*


WORKDIR /app


# we will add cargo files and run cargo build command to get all the 
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build 

RUN rm -rf src

COPY . .
EXPOSE 8080
CMD ["cargo" , "run"]
