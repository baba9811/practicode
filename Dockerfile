FROM rust:1-bookworm AS rust
FROM node:22-bookworm-slim AS node
RUN npm install --global typescript@5.9.3 \
  && npm cache clean --force
FROM eclipse-temurin:21-jdk-jammy AS java

FROM python:3.12-bookworm

RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=rust /usr/local/cargo /usr/local/cargo
COPY --from=rust /usr/local/rustup /usr/local/rustup
COPY --from=node /usr/local/bin/node /usr/local/bin/node
COPY --from=node /usr/local/lib/node_modules/typescript /usr/local/lib/node_modules/typescript
COPY --from=java /opt/java/openjdk /opt/java/openjdk

ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV JAVA_HOME=/opt/java/openjdk
ENV PATH=/opt/java/openjdk/bin:/usr/local/cargo/bin:$PATH

RUN set -eu; \
  ln -s /usr/local/lib/node_modules/typescript/bin/tsc /usr/local/bin/tsc; \
  ln -s /opt/java/openjdk/bin/java /usr/local/bin/java; \
  ln -s /opt/java/openjdk/bin/javac /usr/local/bin/javac; \
  ln -s /usr/local/cargo/bin/cargo /usr/local/bin/cargo; \
  ln -s /usr/local/cargo/bin/rustc /usr/local/bin/rustc; \
  python_version="$(python3 --version 2>&1)"; \
  node_version="$(node --version)"; \
  typescript_version="$(tsc --version)"; \
  java_version="$(javac -version 2>&1)"; \
  case "$python_version" in "Python 3.12."*) ;; *) exit 1 ;; esac; \
  case "$node_version" in "v22."*) ;; *) exit 1 ;; esac; \
  test "$typescript_version" = "Version 5.9.3"; \
  case "$java_version" in "javac 21."*) ;; *) exit 1 ;; esac; \
  printf '%s\n%s\n%s\n%s\n' "$python_version" "$node_version" "$typescript_version" "$java_version"; \
  rustc --version

WORKDIR /opt/practicode
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY assets ./assets
RUN cargo build --release --locked \
  && install -m 0755 target/release/practicode /usr/local/bin/practicode \
  && rm -rf target /usr/local/cargo/registry /usr/local/cargo/git

ENV HOME=/tmp
ENV PRACTICODE_NO_UPDATE_CHECK=1
WORKDIR /workspace
ENTRYPOINT ["practicode"]
