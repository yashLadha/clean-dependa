<p align="center"> <img src="http://ForTheBadge.com/images/badges/made-with-rust.svg" /> </p>

## clean-dependa 🧹

Clean-Dependa is a hobby project to learn rust and create something that is
bothering me from such a long time 😓. It looks for all the user-repositories and
clears up the dependabot PRs of high severity.

Behind the scenes it is using the awesome 🤘Github API to perform these tasks.

## Building

Clone the repository and in the root directory execute the following command.

```sh
cargo build --release
```

## Usage

You can use the build binary in release folder inside target directory. You need
to pass your Github Username as the command line argument and set the
environment variable `CLONE_TOKEN` to the github token value.

```sh
export GITHUB_TOKEN="<your_github_token>"
./target/release/clean-dependa yashladha
```

## Enhancements Pending

* To create PRs of all levels from Github and not just critical severity.
* Management of different orgs and not just the user.
