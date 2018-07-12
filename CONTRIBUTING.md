# Contributing

## Filing an Issue

If you are trying to use this application and its libraries and run into an
issue - please file an issue! We'd love to get you up and running, even if the
issue you have might not be directly be related to this project's own code base.

When filing an issue, do your best to be as specific as possible. Use a short
and concise title that fits into a single line. Like any good commit message
titles don't end with a period ;) Add a comprehensive description that explains
your motivation and includes steps to reproduce exceptional behaviour in case
of a bug.

## Writing Code

### Use the `stable` Toolchain

Use the `rustup override` command to make sure that you are using the `stable`
toolchain. Run this command in the `aoide-rs` directory you cloned.

```sh
rustup override set stable
```

### Keep Rust and all Components up-to-date

We closely follow all updates of `stable` Rust and its components. Keep your installation up-to-date with the
following command:

```sh
rustup update stable
```

### Format the Code

Before submitting code in a PR, make sure that you have formatted the codebase using
[rustfmt](rustfmt). `rustfmt` is a tool for formatting Rust code, which helps keep style
consistent across the project.

If you have not already configured `rustfmt` for the stable toolchain, install the most recent
version of `rustfmt` using this command:

```sh
rustup component add rustfmt-preview --toolchain stable
```

To run `rustfmt`, use this command:

```sh
cargo fmt
```

[rustfmt]: https://github.com/rust-lang-nursery/rustfmt/

### Follow the API Guidelines

All code written in Rust should follow the [Rust API Guidelines].

Use existing code as a template, but do **not** copy it blindly. Remember
that code, even when written by the most experienced contributor long ago,
might not comply with the current version of the guide. When in doubt
don't hesitate to ask for help or advice.

[Rust API Guidelines]: https://rust-lang-nursery.github.io/api-guidelines/

### Committing Changes

Keep your change sets small per commit:
* Revert any unnecessary changes or changes to unrelated files
* Use separate commits when moving or renaming files and when fixing formatting issues with `rustfmt`
* Don't accidentilly check in any temporary files

Make sure that the code compiles without errors. The only exeception is allowed for tests that might neither compile
nor succeed for an intermediate commit.

Try *really* hard to follow [The seven rules of a great Git commit message]:

1. Separate subject from body with a blank line
2. Limit the subject line to 50 characters
3. Capitalize the subject line
4. Do not end the subject line with a period
5. Use the imperative mood in the subject line
6. Wrap the body at 72 characters
7. Use the body to explain what and why vs. how

[The seven rules of a great Git commit message]: https://chris.beams.io/posts/git-commit/

## Submitting a Pull Request (PR)

If you are considering to submit a pull request, make sure that there's an issue
filed for the work you'd like to do. There might be some discussion required!
Filing an issue first helps to ensure that the work you put into your pull
request is acceptable for all participants of the project and will get merged :)

Before you submit your pull request, check that you have completed all of the
steps and followed the rules mentioned above in *Writing Code*. Finally link
the issue that your pull request is responding to in the descriptive text of
the PR.

## Conduct

We follow the Rust [Code of Conduct].

[Code of Conduct]: https://www.rust-lang.org/conduct.html