ghz 127.0.0.1:8090 \
  --insecure \
  --proto proto/vino.proto \
  --call vino.InvocationService.Invoke \
  --data-file ../../etc/grpc-payloads/wapc-invoke.json

ghz 127.0.0.1:8060 --insecure --proto crates/vino-rpc/proto/vino.proto --call vino.InvocationService.Invoke --data-file ./etc/grpc-payloads/echo.json -n 3000

echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid
perf record --call-graph=dwarf ./build/local/vino start tests/manifests/echo.yaml --port 8060
then
ghz 127.0.0.1:8060 --insecure --proto crates/vino-rpc/proto/vino.proto --call vino.InvocationService.Invoke --data-file ./etc/grpc-payloads/echo.json -n 3000