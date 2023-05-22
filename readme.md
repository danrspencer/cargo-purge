# Cargo Purge (Alpha Version)

**Please note that Cargo Purge is currently in early alpha development. Use it with caution and expect potential issues or limitations. Your feedback and contributions are highly appreciated.**

Cargo Purge is a command-line tool for analyzing a Rust workspace and identifying publicly exported items from workspace packages that are not used within the workspace. Its purpose is to help you find and eliminate dead code across your workspace packages.

Please note that Cargo Purge is intended for internal use within a Rust workspace and should not be used if the libraries are public outside of the workspace. It analyzes the workspace's dependencies and does not consider external dependencies or downstream projects.

## Installation

To install Cargo Purge, ensure that you have Rust and Cargo installed on your system. You can install the tool from the repository by following these steps:

1. Clone the repository to your local machine:

   ```
   $ git clone https://github.com/your-username/cargo-purge.git
   ```

2. Navigate to the cloned repository:

   ```
   $ cd cargo-purge
   ```

3. Build and install the tool using Cargo:

   ```
   $ cargo install --path .
   ```

   This command will build the tool and install it into your system's binary directory.

## Usage

Once Cargo Purge is installed, you can run it from the root directory of your Rust workspace using the following command:

```
$ cargo purge
```

Cargo Purge will scan all the packages within your workspace and identify any publicly exported items that are not used within the workspace. It will provide you with a report listing the unused items, allowing you to assess and remove them as necessary.

## Configuration

Cargo Purge does not require any configuration to run, but it respects the configuration files of your Rust workspace. If you have a `Cargo.toml` file in your workspace's root directory or in any of the packages, Cargo Purge will consider it when analyzing the dependencies and determining which items are unused.

## Limitations

- Cargo Purge only analyzes the workspace's dependencies and does not consider external dependencies or downstream projects. It is designed for internal use within a Rust workspace.
- Cargo Purge may produce false positives or false negatives in certain cases. It relies on static analysis and may not accurately detect all unused code.
- Cargo Purge should not be used if the libraries are public outside of the workspace. It is not intended for analyzing public APIs.

## Contributing

If you encounter any issues or have suggestions for improving Cargo Purge, please feel free to open an issue or submit a pull request on the [GitHub repository](https://github.com/your-username/cargo-purge). Contributions are welcome!

## License

Cargo Purge is open source software licensed under the [MIT License](https://opensource.org/licenses/MIT). You can find the full text of the license in the `LICENSE` file in the project repository.

## Acknowledgments

Cargo Purge is inspired by the need to identify and eliminate dead code within Rust workspaces. Thank you to the Rust community for providing a rich ecosystem and tools to improve code quality.

## Contact

For any further questions or inquiries, please contact [your-email@example.com](mailto:your-email@example.com).

---

**Note:** Replace `your-username` in the GitHub repository URL with your actual GitHub username.