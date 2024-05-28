You made it here! This is a great step in helping to contribute to ARK 🎈

## How to contribute

To get started, you can start off here [issues](https://github.com/ARK-Builders/ark-rust/issues) with those tagged [`good first issue`](https://github.com/ARK-Builders/ark-rust/issues?q=is:issue+is:open+label:%22good+first+issue%22).

You can find fresh builds as artifacts of [GitHub Actions workflows](https://github.com/ARK-Builders/ark-rust/actions):

- The "Verify build" workflow runs tests on supported platforms
- Benchmarks are run on every PR. It uses `criterion` to measure performance of the code compared to current main branch

## Forking the project

Before we can add you as a contributor to our project, we suggest to do initial work from your own fork of the project.

To create a fork, please press `fork` button on the project page:
![contr1](https://user-images.githubusercontent.com/581023/162485594-27755479-8509-4d4b-8983-54980d899c50.png)

Then you can modify everything without fear of breaking official version.

## Submitting a Pull Request

After you've implemented a feature or fixed a bug, it is time to open Pull Request.
![contr2](https://user-images.githubusercontent.com/581023/162485618-d8d447b9-591f-41c8-ab3d-1ceb61090ca3.png)

Please enable GitHub Actions in your fork, so our QA will be able to download build of your version without manually compiling from source code.
![contr3](https://user-images.githubusercontent.com/581023/162485639-3d35b8fe-6808-4983-a480-41b65a1ce9b2.png)

### Automated code style checks

The projects embeds `ktlint` in order to enforce consistent code style.

Before a PR can be merged, you would have to fix all code style errors.

### Code review

We care a lot about our software quality, that's why we are conducting strict code reviews before merging:

- we will ask questions if we are not sure about particular technical decision
- when possible, we will suggest alternative solution
- GitHub Actions workflow must result in success (be green)
- comments must be resolved before merge
- code style should be green as well

Right now, the team isn't that big, so please be patient 🙂

### Merge conflicts

If Pull Request is long time in reviewing phase, `main` branch might go forward too far.
Please, fix all merge conflicts in this case 🛠

## Additional read

https://docs.github.com/en/get-started/quickstart/github-flow
