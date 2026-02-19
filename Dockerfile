FROM rust:latest as build

WORKDIR /usr/src/gradwork-backend
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=build /usr/src/gradwork-backend/target/release/gradwork-backend /usr/local/bin/gradwork-backend

WORKDIR /usr/local/bin
CMD ["gradwork-backend"]
