FROM debian:bullseye-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y libssl1.1 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY docker-files/duolang /app/duolang

RUN chmod +x /app/duolang

ENV RUST_LOG=info

EXPOSE 3000

CMD ["./duolang"]
