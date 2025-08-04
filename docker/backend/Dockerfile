# Builder Stage
FROM rust:latest AS builder

WORKDIR /opt/lilac/api

# Install the application dependencies
COPY ./backend/src src
COPY ./backend/migrations migrations
COPY ./backend/.sqlx .sqlx
COPY ./backend/Cargo.toml Cargo.toml
COPY ./backend/Cargo.lock Cargo.lock

RUN mkdir build/

# Set SQLX to offline mode for the build
ENV SQLX_OFFLINE=true
RUN cargo build --package server --target-dir ./build/

# Final Stage
FROM rust:latest

WORKDIR /opt/lilac/api

ENV LILAC_CONFIG_FILE=./lilac.toml

COPY --from=builder /opt/lilac/api/build/debug/server ./server
COPY ./backend/lilac.toml ./lilac.toml
RUN groupadd --system appgroup && \
    useradd --system --gid appgroup --no-create-home appuser

USER appuser

EXPOSE 8081

CMD [ "./server" ]