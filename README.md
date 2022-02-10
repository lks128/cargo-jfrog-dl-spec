# cargo-jfrog-dl-spec

A cargo subcommand that based on a Cargo.lock file generates JFrog CLI spec file to download crates from a private registry.

## Install
```shell
cargo install cargo-jfrog-dl-spec
```

## Usage
```shell
# Generate spec file
cargo jfrog-dl-spec -c artifactory -j cargo-private >spec.json

# Download crates from spec file
jfrog rt dl --spec=spec.json
```