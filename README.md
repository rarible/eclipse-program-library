# Eclipse Program Library Testing Guide

This guide provides step-by-step instructions for setting up and running tests for the Eclipse Program Library.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools)
- [Anchor](https://www.anchor-lang.com/docs/installation)
- [Node.js and npm](https://nodejs.org/en/download/)
- [Yarn](https://classic.yarnpkg.com/en/docs/install)

## Setup and Testing Steps

1. Clone the repository:
   ```
   git clone https://github.com/rarible/eclipse-program-library.git
   cd eclipse-program-library
   ```

2. Install dependencies:
   ```
   yarn install
   ```

3. Sync program IDs:
   - Open `Anchor.toml` and `constants.ts`
   - Ensure that the program IDs in both files match
   - Example:
     ```toml
     # In Anchor.toml
     [programs.localnet]
     rarible_editions = "587DoLBH2H39i5bToWBc6zRgbD2iJZtc4Kb8nYsskYTq"
     rarible_editions_controls = "5hEa5j38yNJRM9vQA44Q6gXVj4Db8y3mWxkDtQeofKKs"
     ```
     ```typescript
     // In constants.ts
     export const rarible_editions_PROGRAM_ID = new PublicKey("587DoLBH2H39i5bToWBc6zRgbD2iJZtc4Kb8nYsskYTq");
     export const rarible_editions_CONTROLS_PROGRAM_ID = new PublicKey("5hEa5j38yNJRM9vQA44Q6gXVj4Db8y3mWxkDtQeofKKs");
     ```

4. Run tests:
   - To build and run tests:
     ```
     anchor test
     ```
   - To skip building and only run tests (faster for repeated testing):
     ```
     anchor test --skip-build
     ```

## Notes

- If you encounter wallet-related errors, ensure your `Anchor.toml` is configured to use your local keypair:
  ```toml
  [provider]
  cluster = "localnet"
  wallet = "~/.config/solana/id.json"
  ```

- For any issues or questions, please open an issue in the GitHub repository.

## Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) file for details on how to contribute to this project.

