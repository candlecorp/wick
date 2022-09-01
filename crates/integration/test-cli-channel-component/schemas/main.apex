namespace "main"

alias Link = string

type Inputs {
  args: [string]
  isInteractive: Interactive
  program: Link?
}

type Interactive {
  stdin: bool
  stdout: bool
  stderr: bool
}

type Outputs {
  code: u32
}

