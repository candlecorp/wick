package main

import (
	"encoding/json"
	"flag"
	"log"
	"time"

	"github.com/nats-io/nats.go"
	"github.com/vmihailenco/msgpack/v5"
)

func main() {
	flag.Parse()
	args := flag.Args()
	if len(args) != 4 {
		log.Fatal("usage: nats-req <namespace> <service> <operation> <data>")
	}

	ns := args[0]
	service := args[1]
	operation := args[2]
	data := args[3]

	// Connect to a server
	nc, err := nats.Connect(nats.DefaultURL)
	if err != nil {
		log.Fatal(err)
	}

	subject := ns + "." + service + "." + operation
	log.Println(subject)
	request := nats.NewMsg(subject)
	request.Header.Set("Namespace", ns)
	request.Header.Set("Service", service)
	request.Header.Set("Function", operation)

	var payload interface{}
	if err := json.Unmarshal([]byte(data), &payload); err != nil {
		log.Fatal(err)
	}

	request.Data, err = msgpack.Marshal(&payload)
	if err != nil {
		log.Fatal(err)
	}
	request.Header.Set("Content-Type", "application/msgpack")
	reply, err := nc.RequestMsg(request, 5*time.Second)
	if err != nil {
		log.Fatal(err)
	}

	for k, v := range reply.Header {
		log.Println(k, "=", v)
	}

	payload = nil
	jsonBytes := reply.Data
	if err = msgpack.Unmarshal(reply.Data, &payload); err == nil {
		jsonBytes, err = json.Marshal(&payload)
		if err != nil {
			log.Fatal(err)
		}
	}

	log.Println(string(jsonBytes))
}
