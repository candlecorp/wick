ghz 127.0.0.1:8090 \
  --insecure \
  --proto proto/vino.proto \
  --call vino.InvocationService.Invoke \
  --data-file ../../etc/grpc-payloads/wapc-invoke.json
