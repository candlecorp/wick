package stateful

type RawItem struct {
	Namespace  string `json:"namespace,omitempty" msgpack:"namespace,omitempty"`
	Type       string `json:"type,omitempty" msgpack:"type,omitempty"`
	Version    string `json:"version,omitempty" msgpack:"version,omitempty"`
	Data       []byte `json:"data,omitempty" msgpack:"data,omitempty"`
	DataBase64 string `json:"dataBase64,omitempty" msgpack:"dataBase64,omitempty"`
}

type Mutation struct {
	Set    map[string]RawItem `json:"set,omitempty" msgpack:"set,omitempty"`
	Remove []string           `json:"remove,omitempty" msgpack:"remove,omitempty"`
}

type Response struct {
	Mutation Mutation    `json:"mutation,omitempty" msgpack:"mutation,omitempty"`
	Result   interface{} `json:"result,omitempty" msgpack:"result,omitempty"`
}
