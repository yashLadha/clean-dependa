<p align="center"> <img src="http://ForTheBadge.com/images/badges/made-with-rust.svg" /> </p>

## clean-dependa 🧹

Clean-Dependa is a hobby project to learn rust and create something that is
bothering me from such a long time 😓. It looks for all the user-repositories and
clears up the dependabot PRs of high severity.

Behind the scenes it is using the awesome 🤘Github API to perform these tasks.

## Building

Clone the repository and in the root directory execute the following command.

```sh
make
```

You can find the binary at path `target/release/clean-dependa`. Move it to a
location which is visible in your `$PATH`.

## Usage

You can use the build binary in release folder inside target directory. You need
to pass your Github Username as the command line argument and set the
environment variable `GITHUB_TOKEN` to the github token value.

```sh
export GITHUB_TOKEN="<your_github_token>"
clean-dependa yashladha
```

You can also copy the binary in your `$PATH` so that it can be accessible from
all the paths.

## Enhancements Pending

* To create PRs of all levels from Github and not just critical severity.
* Management of different orgs and not just the user.
