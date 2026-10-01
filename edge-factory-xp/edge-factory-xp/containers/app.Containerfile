# The Rust application image that runs as a Cloudflare Container behind the Worker shell.
# Built and pushed by `wrangler containers push` / `wrangler deploy` from the app repo;
# here as the reference shape. Build context: repository root.
FROM docker.io/library/rust:1.90-bookworm AS build
WORKDIR /src
COPY app/ .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /src/target/release/app /usr/local/bin/app
EXPOSE 8080
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/app"]
