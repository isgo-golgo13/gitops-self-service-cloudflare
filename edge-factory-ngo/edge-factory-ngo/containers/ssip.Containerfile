# edgefactory-ssip image. FROM rust: must equal the local toolchain (rustc --version).
FROM docker.io/library/rust:1.98-bookworm AS build
WORKDIR /src
COPY ssip/ .
RUN cargo build --release -p edgefactory-ssip

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /src/target/release/edgefactory-ssip /usr/local/bin/edgefactory-ssip
# cosign is invoked as a subprocess for freight signing.
COPY --from=gcr.io/projectsigstore/cosign:v3.1.3 /ko-app/cosign /usr/local/bin/cosign
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/edgefactory-ssip"]
