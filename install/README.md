# NanoBus Installation

## Windows

### Get the latest stable version

```
powershell -Command "iwr -useb https://raw.githubusercontent.com/nanobus/nanobus/main/install/install.ps1 | iex"
```

### Get a specific version

```
powershell -Command "$script=iwr -useb https://raw.githubusercontent.com/nanobus/nanobus/main/install/install.ps1; $block=[ScriptBlock]::Create($script); invoke-command -ScriptBlock $block -ArgumentList <Version>"
```

## MacOS

### Get the latest stable version

```
curl -fsSL https://raw.githubusercontent.com/nanobus/nanobus/main/install/install.sh | /bin/bash
```

### Get a specific version

```
curl -fsSL https://raw.githubusercontent.com/nanobus/nanobus/main/install/install.sh | /bin/bash -s <Version>
```

## Linux

### Get the latest stable version

```
wget -q https://raw.githubusercontent.com/nanobus/nanobus/main/install/install.sh -O - | /bin/bash
```

### Get a specific version

```
wget -q https://raw.githubusercontent.com/nanobus/nanobus/main/install/install.sh -O - | /bin/bash -s <Version>
```
