# AWS Terminal Service (ATS)

ATS is a terminal-based UI for managing AWS services, inspired by k9s for
Kubernetes. Built with Rust and Ratatui, it provides an efficient interface for
interacting with various AWS services.

## Architecture

### Core Components

- **Header**: Shows application info, AWS profile, region, and status
- **Input Bar**: Dual-mode input for commands (`:`) and search (`/`)
- **Main Content**: Service-specific resource tables and details
- **Footer**: Status information and keyboard shortcuts

### Input Modes

- **Command Mode** (`:` key): Service navigation and application commands
- **Search Mode** (`/` key): Real-time filtering of current view

## Commands

### Service Commands (k9s style)

- `:ecr` - Switch to ECR repositories view
- (WIP) `:route53` - Switch to Route53 hosted zones view
- (WIP) `:elb` - Switch to ELB load balancers view
- (WIP) `:ec2` - Switch to EC2 instances view
- (WIP) `:s3` - Switch to S3 buckets view

### Application Commands

- `:quit` or `:q` - Quit application
- `:help` or `:?` - Show help screen
- `:refresh` or `:r` - Refresh current view

### Navigation

- `Enter` - Select/drill down into resource
- `Esc` - Go back/cancel current operation
- `Arrow Keys` - Navigate table rows
- `Tab` - Navigate between UI panels

## Development

### Current Project Structure

```
ats/
├── Cargo.toml           # Project dependencies and metadata
├── CLAUDE.md            # Development documentation
├── src/
│   ├── main.rs          # Application entry point and main loop
│   ├── app.rs           # Core application state and logic
│   ├── ui/              # UI components
│   │   ├── mod.rs       # UI module exports
│   │   ├── layout.rs    # Main 4-panel layout management
│   │   ├── header.rs    # Header with app info, profile, region
│   │   ├── input.rs     # Dual-mode input bar (:command, /search)
│   │   ├── content.rs   # Main content area with table rendering
│   │   └── footer.rs    # Status bar and hotkey hints
│   ├── services/        # AWS service implementations
│   │   ├── mod.rs       # Services module exports
│   │   └── ecr.rs       # ECR service integration (implemented)
│   └── utils/           # Utility functions
│       ├── mod.rs       # Utils module exports
│       └── aws.rs       # AWS SDK client creation and config
└── target/              # Cargo build artifacts
```

### Implementation Status

**✅ Completed:**

- Core TUI framework with 4-panel layout
- ECR service integration with repository listing
- Command mode (`:ecr`, `:quit`, `:refresh`)
- Search/filter mode (`/pattern`)
- Keyboard navigation (arrows, Enter, Esc)
- Error handling and loading states
- AWS credential chain integration
- Press `c` to copy selected resource's info to clipboard

**🚧 Planned (WIP in documentation):**

- Additional AWS services (EC2, Route53, ELB, S3, Lambda, CloudWatch)
- Help system (`:help`)
- Resource detail drilling
- Resource operations (start/stop, etc.)

### Dependencies

**Core Framework:**

- `ratatui` v0.29.0 - Terminal UI framework
- `crossterm` v0.29.0 - Cross-platform terminal handling
- `tokio` v1.0 - Async runtime with full features

WebFetch [ratatui website](https://ratatui.rs/) and
[ratatui docs](https://docs.rs/ratatui/latest/ratatui/index.html) when need help
while using `ratatui` framework.

**AWS Integration:**

- `aws-config` - AWS configuration management
- `aws-sdk-ecr` - ECR service SDK (currently implemented)

**Utilities:**

- `serde` - Serialization with derive features
- `anyhow` - Error handling
- `chrono` - Date/time handling with serde support

### Testing Commands

Run below commands to test the application:

- `cargo check` - Check code for errors
- `cargo build` - Build the application
- `cargo clippy` - Run linting
- `cargo fmt` - Format code

### AWS Configuration

The application uses the standard AWS credential chain:

1. Environment variables:
   - `AWS_PROFILE` - Set AWS profile (defaults to "default")
   - `AWS_REGION` / `AWS_DEFAULT_REGION` - Set region (defaults to "us-east-1")
   - `AWS_ACCESS_KEY_ID` / `AWS_SECRET_ACCESS_KEY` - Direct credentials
2. AWS credentials file (`~/.aws/credentials`)
3. AWS config file (`~/.aws/config`)
4. IAM roles (when running on EC2)

**Example Usage:**

```bash
# Use specific profile and region
cargo run -p my-profile -r eu-west-1

# Use default profile with specific region
cargo run -r us-west-2
```

## Color Scheme

- **Red**: for errors

- For selected row items, `Style::default().bg(Color::Yellow).fg(Color::Black)`
  which looks like AWS primary color scheme.

## Design Principles

- **Efficiency**: Fast navigation with minimal keystrokes
- **Consistency**: Similar to k9s command patterns
- **Clarity**: Clear visual hierarchy and status indicators
- **Performance**: Async operations with responsive UI
