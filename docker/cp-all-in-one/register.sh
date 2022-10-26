#curl -X DELETE http://localhost:8081/subjects/customers

curl -X POST -H "Content-Type: application/vnd.schemaregistry.v1+json" \
  -d "@schema.avro" \
  http://localhost:8081/subjects/customers/versions