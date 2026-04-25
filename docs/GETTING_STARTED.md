# Getting Started Guide for Contributors

Welcome to VoteChain! This guide will help you set up your development environment and make your first contribution.

## Prerequisites

Before you begin, ensure you have the following installed on your system:

### Required Software

| Tool | Minimum Version | Purpose |
|------|----------------|---------|
| **Rust** | 1.75.0+ | Core language for smart contracts |
| **Cargo** | 1.75.0+ | Rust package manager (comes with Rust) |
| **Stellar CLI** | Latest | Build and deploy Soroban contracts |
| **Git** | 2.0+ | Version control |

### Optional but Recommended

| Tool | Purpose |
|------|---------|
| **VS Code** | IDE with Rust extensions |
| **rust-analyzer** | Rust language server for IDE support |

## Environment Setup

### Step 1: Install Rust

Install Rust using rustup (the official Rust installer):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Follow the on-screen instructions. After installation, restart your terminal and verify:

```bash
rustc --version
cargo --version
```

### Step 2: Add WebAssembly Target

Soroban contracts compile to WebAssembly. Add the wasm32 target:

```bash
rustup target add wasm32-unknown-unknown
```

### Step 3: Install Stellar CLI

The Stellar CLI is required to build and deploy Soroban contracts:

```bash
cargo install --locked stellar-cli --features opt
```

Verify the installation:

```bash
stellar --version
```

### Step 4: Clone the Repository

Clone the VoteChain repository and navigate to the project directory:

```bash
git clone https://github.com/Vera3289/votechain-contracts.git
cd votechain-contracts
```

### Step 5: Build the Project

Build both contracts (governance and token):

```bash
make build
```

This compiles the contracts to `.wasm` files in the `target/wasm32-unknown-unknown/release/` directory.

### Step 6: Run Tests

Run the full test suite to ensure everything is working:

```bash
make test
```

You should see output indicating all tests passed:

```
running 45 tests
test test_create_proposal ... ok
test test_cast_vote_and_finalise_passed ... ok
...
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Running Tests

### Run All Tests

```bash
make test
```

### Run Tests for a Specific Contract

```bash
# Governance contract only
cargo test -p votechain-governance

# Token contract only
cargo test -p votechain-token
```

### Run a Specific Test

```bash
cargo test test_create_proposal
```

### Run Tests with Output

By default, Rust captures test output. To see println! statements:

```bash
cargo test -- --nocapture
```

### Run Tests in Verbose Mode

```bash
cargo test -- --test-threads=1 --nocapture
```

## Code Quality Checks

Before submitting a pull request, ensure your code passes all quality checks:

### Format Check

```bash
make fmt-check
```

If formatting issues are found, auto-fix them:

```bash
make fmt
```

### Linting

Run Clippy (Rust's linter) to catch common mistakes:

```bash
make lint
```

Fix any warnings or errors before committing.

## Deploying to Testnet

### Step 1: Configure Testnet

Ensure you have a Stellar testnet account with XLM for fees. You can get testnet XLM from the [Stellar Laboratory](https://laboratory.stellar.org/#account-creator?network=test).

### Step 2: Set Environment Variables

Create a `.env` file in the project root (this file is gitignored):

```bash
# Stellar testnet configuration
STELLAR_NETWORK=testnet
STELLAR_RPC_URL=https://soroban-testnet.stellar.org
STELLAR_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"

# Your testnet account secret key (NEVER commit this!)
STELLAR_SECRET_KEY=S...
```

### Step 3: Deploy Contracts

Deploy the token contract first, then the governance contract:

```bash
NETWORK=testnet ./scripts/deploy.sh
```

The script will:
1. Build both contracts
2. Deploy the token contract
3. Deploy the governance contract
4. Initialize both contracts
5. Output the deployed contract addresses

### Step 4: Verify Deployment

Check the deployment on [Stellar Expert](https://stellar.expert/explorer/testnet) by searching for your contract addresses.

## Making Your First Contribution

### Step 1: Create a Branch

Create a new branch for your feature or bug fix:

```bash
git checkout -b feature/my-new-feature
```

Use descriptive branch names:
- `feature/add-delegation`
- `fix/double-vote-bug`
- `test/quorum-edge-cases`
- `docs/update-readme`

### Step 2: Make Your Changes

Edit the relevant files. Common areas:

| Area | Files |
|------|-------|
| Governance logic | `contracts/governance/src/lib.rs` |
| Token logic | `contracts/token/src/lib.rs` |
| Events | `contracts/*/src/events.rs` |
| Tests | `contracts/*/src/test.rs` |
| Documentation | `README.md`, `docs/` |

### Step 3: Write Tests

Every new function or bug fix should include tests. Add tests to the appropriate `test.rs` file:

```rust
#[test]
fn test_my_new_feature() {
    let t = setup_env();
    // Your test code here
    assert_eq!(expected, actual);
}
```

### Step 4: Run Quality Checks

Before committing, ensure all checks pass:

```bash
make test
make fmt
make lint
```

### Step 5: Commit Your Changes

Follow [Conventional Commits](https://www.conventionalcommits.org/) format:

```bash
git add .
git commit -m "feat: add delegation support to governance contract"
```

Commit message prefixes:
- `feat:` - New feature
- `fix:` - Bug fix
- `test:` - Adding or updating tests
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `chore:` - Maintenance tasks

### Step 6: Push and Create a Pull Request

Push your branch to GitHub:

```bash
git push origin feature/my-new-feature
```

Then create a pull request on GitHub with:
- Clear title describing the change
- Description of what was changed and why
- Reference to any related issues (e.g., "Closes #42")
- Screenshots or examples if applicable

## Common Troubleshooting

### Issue: `cargo test` fails with "could not compile"

**Solution:** Ensure you're using Rust 1.75.0 or later:

```bash
rustup update
```

### Issue: `stellar: command not found`

**Solution:** Reinstall the Stellar CLI:

```bash
cargo install --locked stellar-cli --features opt
```

Ensure `~/.cargo/bin` is in your PATH.

### Issue: `wasm32-unknown-unknown` target not found

**Solution:** Add the WebAssembly target:

```bash
rustup target add wasm32-unknown-unknown
```

### Issue: Tests fail with "already initialized" error

**Solution:** This is expected behavior for re-initialization tests. If other tests fail, ensure you're running the latest code:

```bash
git pull origin main
cargo clean
make build
make test
```

### Issue: Deployment fails with "insufficient balance"

**Solution:** Ensure your testnet account has XLM. Get testnet XLM from:
- [Stellar Laboratory](https://laboratory.stellar.org/#account-creator?network=test)
- [Friendbot](https://friendbot.stellar.org/)

### Issue: `make build` fails with linker errors

**Solution:** Ensure you have the correct Rust toolchain:

```bash
rustup default stable
rustup target add wasm32-unknown-unknown
```

### Issue: Clippy warnings about unused imports

**Solution:** Remove unused imports or allow them temporarily:

```rust
#[allow(unused_imports)]
use soroban_sdk::...;
```

### Issue: Format check fails

**Solution:** Auto-format your code:

```bash
make fmt
```

## Project Structure

Understanding the project layout:

```
votechain-contracts/
├── contracts/
│   ├── governance/          # Governance contract
│   │   ├── src/
│   │   │   ├── lib.rs       # Main contract logic
│   │   │   ├── storage.rs   # Storage helpers
│   │   │   ├── events.rs    # Event emissions
│   │   │   ├── types.rs     # Type definitions
│   │   │   └── test.rs      # Unit tests
│   │   └── Cargo.toml       # Contract dependencies
│   └── token/               # Token contract (similar structure)
├── docs/                    # Documentation
│   ├── adr/                 # Architecture Decision Records
│   ├── examples/            # Usage examples
│   └── security/            # Security documentation
├── scripts/                 # Deployment scripts
├── config/                  # Network configurations
├── Cargo.toml               # Workspace configuration
├── Makefile                 # Build commands
└── README.md                # Project overview
```

## Development Workflow

1. **Pick an issue** from the GitHub issues page or create a new one
2. **Create a branch** with a descriptive name
3. **Write code** following the project's style and conventions
4. **Write tests** for your changes
5. **Run quality checks** (test, fmt, lint)
6. **Commit** with a conventional commit message
7. **Push** and create a pull request
8. **Respond to feedback** from maintainers
9. **Merge** once approved

## Coding Standards

- **No `std`:** All contracts use `#![no_std]`
- **Error handling:** Use `Result<T, ContractError>` for fallible operations
- **Events:** Emit events for all state-changing operations
- **Documentation:** Add `///` doc comments to all public functions
- **Testing:** Every function needs at least one test
- **No floating-point:** Use `i128` for all numeric values
- **Formatting:** Run `make fmt` before committing
- **Linting:** Fix all Clippy warnings

## Getting Help

- **GitHub Issues:** [github.com/Vera3289/votechain-contracts/issues](https://github.com/Vera3289/votechain-contracts/issues)
- **Stellar Discord:** [discord.gg/stellar](https://discord.gg/stellar) - #soroban channel
- **Stellar Docs:** [developers.stellar.org/docs/smart-contracts](https://developers.stellar.org/docs/smart-contracts)
- **Soroban Examples:** [github.com/stellar/soroban-examples](https://github.com/stellar/soroban-examples)

## Next Steps

Now that you're set up:

1. Read the [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines
2. Review the [Architecture Decision Records](adr/README.md) to understand design choices
3. Check the [FAQ](faq.md) for common questions
4. Browse [open issues](https://github.com/Vera3289/votechain-contracts/issues) to find something to work on
5. Join the Stellar Discord to connect with the community

Happy coding! 🚀
