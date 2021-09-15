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

# Bugs

- Component codegen for wellknown providers doesn't reference the proper module for input/outputs etc
- interface.json's for wellknown interfaces needs to be located in a central repo
- Type representation of components over RPC is just a string and needs to be more complex to represent valid types.

## Error codes

### 1XXX

### 2XXX

### 3XXX

### 4XXX

### 5XXX Network Error

### 6XXX Schematic Error

### 7XXX Transaction Error

### 8XXX

### 9XXX

# Doc links

- docs.vino.dev/codegen

## Todos

- Codegen makefiles

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
