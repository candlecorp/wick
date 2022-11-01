FROM gcr.io/distroless/static
COPY nanobus /app/nanobus
WORKDIR /app
ENTRYPOINT ["/app/nanobus"]
