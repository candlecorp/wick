namespace "main"

type Inputs {
  args: [string]
  isInteractive: Interactive
  program: link?
}

type Interactive {
  stdin: bool
  stdout: bool
  stderr: bool
}

type Outputs {
  code: u32
}

