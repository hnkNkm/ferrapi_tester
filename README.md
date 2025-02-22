# ferrapi_tester

FerrAPI Tester is a command-line tool for testing HTTP APIs. It allows you to send requests using various HTTP methods and supports saving and loading configurations via a namespace. You can use it for direct API calls or for storing frequently used configurations for later reuse.

## Features

- **HTTP Request Support:** Send GET, POST, PUT, DELETE, etc. requests.
- **Configuration Saving/Loading:** Save your API configuration (URL, method, headers, JSON body, timeout) under a namespace.
- **Interactive Namespace Selection:** Use the `--comp` option to interactively select a namespace recursively from your configuration directory.
- **Namespace Management:**  
  - Create new namespaces using the `--create-namespace` option.  
  - Delete a specific configuration with `--delete`.  
  - Remove an entire namespace with `--delete-all`.
- **Direct API Calls:** If no namespace (TARGET) is provided, execute the API request directly using the `--url` option.
- **Shell Completions:** Generate shell completion scripts for bash, zsh, fish, etc.

## Installation

### Using Cargo

You can install FerrAPI Tester via Cargo (once published on Crates.io):

#### Building from Source
Clone the repository and build the project:

```bash
cargo install ferrapi_tester
git clone https://github.com/your_username/ferrapi_tester.git
cd ferrapi_tester
cargo build --release
``` 

The built binary will be located in the `target/release` directory.

## Usage

### Direct API Call

To execute a request without saving any configuration, simply provide the `--url` option:

```bash
ferrapi_tester -X POST --url=https://reqres.in/api/users -v '{"name": "morpheus", "job": "leader"}'
``` 

This command sends a `POST` request to the specified URL with the given JSON body.

### Saving and Loading Configuration

To save a configuration under a namespace (TARGET), use the `--save` flag along with a TARGET value:

```bash
ferrapi_tester -X POST --url=https://reqres.in/api/users -v '{"name": "morpheus", "job": "leader"}' --save SystemB/reqres
``` 

This command saves the configuration to a file (e.g., `~/.ferrapi_tester/SystemB/reqres/POST.json`). Later, you can load and use the saved configuration by specifying the same TARGET:

```bash
ferrapi_tester -X POST --url=https://reqres.in/api/users --save SystemB/reqres
``` 

### Interactive Namespace Selection

If you prefer to select a namespace interactively, use the `--comp` option. This launches an interactive prompt that recursively lists all subdirectories under your default configuration directory (`~/.ferrapi_tester`).

For example:

```bash
ferrapi_tester -X POST --comp -u "https://reqres.in/api/users"
``` 

- **Prompt Example:**  
  The tool will first list the top-level namespaces, such as:
Select a namespace in /root/.ferrapi_tester: [0] SystemA [1] SystemB
After selecting one (e.g., SystemB), if subdirectories exist, you will be prompted:
If you choose "Yes", it will display the subdirectories (e.g., "reqres", "test_endpoint") so you can further refine your selection. The final selected namespace (e.g., `SystemB/reqres`) is then used as the TARGET.

### Namespace Management

#### Creating a Namespace

To create a new namespace, use the `--create-namespace` option:

```bash
ferrapi_tester --create-namespace SystemB
``` 

This command creates the namespace directory `~/.ferrapi_tester/SystemB` if it does not already exist.

#### Deleting a Configuration

To delete a saved configuration (for a specific HTTP method) in a namespace, use the `--delete` option:

```bash
ferrapi_tester -X POST --delete SystemB/reqres
``` 

This command deletes the configuration file (e.g., `POST.json`) under `~/.ferrapi_tester/SystemB/reqres`.

#### Deleting an Entire Namespace

To delete an entire namespace directory and all its contents, use the `--delete-all` option:

```bash
ferrapi_tester --delete-all SystemB
``` 

This command removes the entire `SystemB` directory under `~/.ferrapi_tester`.

### Shell Completions

Generate shell completion scripts for your preferred shell using the `--completions` option.

#### Bash Example

```bash
mkdir -p ~/.bash_completion
ferrapi_tester --completions bash > ~/.bash_completion/ferrapi_tester
``` 

Then, add the following to your `~/.bashrc`:

```bash
if [ -f ~/.bash_completion/ferrapi_tester ]; then
  . ~/.bash_completion/ferrapi_tester
fi
``` 

Reload your bash configuration:

```bash
source ~/.bashrc
``` 

### Example Command: Saving a Request

To send a POST request with a JSON body and save the configuration under the namespace `SystemB/reqres`, use:

```bash
ferrapi_tester -X POST -u "https://reqres.in/api/users" -v '{"name": "morpheus", "job": "leader"}' --save SystemB/reqres
``` 

## License

This project is licensed under the MIT License.
