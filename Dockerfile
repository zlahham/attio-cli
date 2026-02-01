FROM rust:1.93-slim

RUN apt-get update && apt-get install -y \
  pkg-config \
  libssl-dev \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

CMD ["cargo", "run"]
