# Contributing to Git Hero

First off, thank you for considering contributing to Git Hero!

## Pull Request Workflow

To maintain repository stability and code quality, direct pushes to the main branch are disallowed. All contributions must be submitted via Pull Requests (PRs). Please follow these steps:

1. **Fork the Repository**: Create your own copy of the repository by clicking the "Fork" button at the top of the GitHub page.
2. **Clone your Fork**: Clone your fork to your local machine:
   ```bash
   git clone https://github.com/YOUR_USERNAME/git-hero.git
   ```
3. **Create a Feature Branch**: Keep your changes organized in a separate branch:
   ```bash
   git checkout -b feature/my-new-feature
   ```
4. **Commit and Push**: Make your changes, commit them with clear messages, and push to your fork:
   ```bash
   git push origin feature/my-new-feature
   ```
5. **Open a Pull Request**: Go to the original Git Hero repository on GitHub and open a Pull Request comparing the original `main` branch with your fork's feature branch.
6. **Code Review**: A maintainer will review your changes and provide feedback. Once approved, your changes will be merged.

## Keeping History Clean

- Write clear, concise commit messages.
- If possible, squash intermediate fix-up commits before opening the PR or during the merge process to keep the history readable.
