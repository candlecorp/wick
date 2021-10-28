# Vino monorepo

## Dependencies

### Root project

make
rust/cargo
node
cargo-deny

### WASM

vino-codegen
tomlq

### Manifest codegen

prettier
widl-template

# TAP

curl https://gitlab.com/esr/tapview/-/raw/master/tapview -o $HOME/.local/bin/tapview && chmod a+x $HOME/.local/bin/tapview

# Bugs

- Component codegen for wellknown providers doesn't reference the proper module for input/outputs etc
- interface.json's for wellknown interfaces needs to be located in a central repo
- Schematics can link a provider that isn't exposed to them, e.g.

```yaml
version: 0
network:
  providers:
    - namespace: perms
      kind: WaPC
      reference: ./build/vino_permissions_s.wasm
    - namespace: permsdb
      kind: Lattice
      reference: authdb
  schematics:
    - name: update-permissions
      providers:
      instances:
        update:
          id: perms::update-permissions
      connections:
        - <> => update[user_id]
        - <> => update[permissions]
        - <link>[permsdb] => update[kv]
        - update[result] => <>
```

-The providerlink `.call(component, args)` is prone to error. It would be great if `args` could include the component so a failure would be less likely.

## Error codes

### 1XXX

### 2XXX

### 3XXX

### 4XXX

### 5XXX Network Error

### 6XXX Schematic Error

### 7XXX Transaction Error

### 8XXX Provider Error

### 9XXX

# Doc links

- docs.vino.dev/codegen

# Need a Makefile primer?

- Check out isaacs's tutorial: https://gist.github.com/isaacs/62a2d1825d04437c6f08
- Your makefiles are wrong: https://tech.davis-hansson.com/p/make/

## Todos

- Create a repo for encoded MessageTransports, TransportMaps, and LatticeRpcMessages to ensure cross-compatibility.
- figure out how to make the wellknown interface.json better (right now it's just copying the generated file)
- Make component codegen for providers using interface.json generated the proper includes.
- Improve coersion for PortOutput/ProviderOutput. Right now you have to know too much about what you're supposed to get.
- ignore cache for :latest OCI requests

## Good first contributions

This is a list of nice-to-haves that would also make good contributions for people looking to get involved with Vino.

### Improving logging & the logger

Logging is a mix of valuable and debug output that would be better if there was a more clear reason for logs of each type.

### Opportunities for code generation

Vino uses code generation extensively and making it better or adding more opportunities to use generated code is usually welcome. Open an issue first to discuss it to be sure that someone isn't already working on it.

### Cleanup & consistency

Anytime you see inconsistencies, feel comfortable taking the recommended action or opening an issue to confirm cleanup is OK.

- Unused error variants : Remove
- Unused dependencies : Remove
- Manifests should use short form syntax when possible.
- Inconsistent naming :
  - "reference" when used for components should be "instance"
  - pluralization
- Inconsistent coding style/idioms across similar modules:
  - Code can be normalized and common code can be extracted, but open an issue to discuss it before jumping in.

### Documentation

Documentation is always helpful.

### Examples

What's your "Hello world"? How do you connect things during experimentation? Those could be great examples!

### Rustdoc examples

Rustdoc examples are always helpful. Examples should be written in a way that they can be copy-pasted and executed as-is without hidden context whenever possible.

### FAQ Documentation

As you go work with Vino, what issues pop up that you solve yourself? Those issues could make good FAQ items.
