# ferrapi_tester

FerrAPI Tester
FerrAPI Tester is a command-line tool for testing HTTP APIs. It allows you to send requests using various HTTP methods and supports saving and loading configurations via a namespace. You can use it for direct API calls or for storing frequently used configurations for later reuse.

## Features
- **HTTP Request Support: Send GET, POST, PUT, DELETE, etc. requests.l
- **Configuration Saving/Loading: Save your API configuration (URL, method, headers, JSON body, timeout) under a namespace.
- **Direct API Calls: If no namespace (TARGET) is provided, execute the API request directly using the --url option.
- *(Upcoming features: Delete configuration and shell completions.)

## Installation
### Using Cargo
You can install FerrAPI Tester via Cargo (once published on Crates.io):

Building from Source
Clone the repository and build the project
```bash cargo install ferrapi_tester
git clone https://github.com/your_username/ferrapi_tester.git
cd ferrapi_tester
cargo build --release
```
The built binary will be located in the target/release directory.


## Usage
### Direct API Call
If you want to execute a request without saving any configuration, simply provide the --url option:

```bash
ferrapi_tester -X POST --url=https://reqres.in/api/users -v '{"name": "morpheus", "job": "leader"}'
```
This command sends a ```POST``` request to the specified URL with the given JSON body.

## Saving and Loading Configuration
To save a configuration under a namespace (TARGET), use the --save flag along with a TARGET 
value:
```bash 
ferrapi_tester -X POST --url=https://reqres.in/api/users -v '{"name": "morpheus", "job": "leader"}' --save SystemB/reqres
```
This command saves the configuration to a file (e.g., ```~/.ferrapi_tester/SystemB/reqres/POST.json```).

Later, you can load and use the saved configuration by specifying the same TARGET:

```bash
ferrapi_tester -X POST SystemB/reqres
```

## Upcoming Features
- Configuration Deletion: Ability to delete saved configuration with ```--delete```.
- Shell Completions: Generate shell completion scripts for bash, zsh, etc.

## License
This project is licensed under the MIT License.
