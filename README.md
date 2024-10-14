# git_phantom

> [!CAUTION]
> This crate is in the early stages of development and is subject to breaking changes.

## What's this?

This app is a tunnel application that allows you to create a temporary shared repository locally and publish it
externally.

## Install

```shell
$ cargo install gph_cli
```

## Usage

### Auth(required)

You need to authenticate oauth2 with your GitHub account.

```shell
$ gph auth
```

### Share your local git repository

Execute the following command on the root of the repository.

```shell
$ gph share [OPTIONS]

Options:
  -r, --repository <REPOSITORY>  Remote repository name
      --no-push                  Don't push local commits to a shared repository
      --readonly                 Forbid other users from pushing to a shared repository
  -h, --help                     Print help
```

## Licence

This crate is licensed under the MIT License or the Apache License 2.0.
