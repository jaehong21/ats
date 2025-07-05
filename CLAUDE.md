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
│   ├── app.rs           # Core application state and logic (refactored)
│   ├── ui/              # UI components
│   │   ├── mod.rs       # UI module exports
│   │   ├── layout.rs    # Main 4-panel layout management
│   │   ├── header.rs    # Header with app info, profile, region
│   │   ├── input.rs     # Dual-mode input bar (:command, /search)
│   │   ├── content.rs   # Generic content renderer (refactored)
│   │   └── footer.rs    # Status bar and hotkey hints
│   ├── services/        # AWS service implementations
│   │   ├── mod.rs       # Services module exports
│   │   ├── traits.rs    # Service framework traits and abstractions
│   │   ├── manager.rs   # Service lifecycle and registry management
│   │   └── ecr.rs       # ECR service plugin implementation
│   └── utils/           # Utility functions
│       ├── mod.rs       # Utils module exports
│       └── aws.rs       # AWS SDK client creation and config
└── target/              # Cargo build artifacts
```

### Implementation Status

**✅ Completed:**

- Core TUI framework with 4-panel layout
- **Service Framework Architecture** (major refactor completed):
  - Generic `AwsService` trait for pluggable service implementations
  - `ServiceManager` for service registration and lifecycle management
  - `ResourceItem` trait with type-safe downcasting support
  - Dynamic command routing based on registered services
- ECR service integration with repository listing (now plugin-based)
- ECR images drill-down with navigation back to repositories
- Command mode (`:ecr`, `:quit`, `:refresh`) with dynamic service discovery
- Search/filter mode (`/pattern`)
- Keyboard navigation (arrows, Enter, Esc) with proper back navigation
- Enhanced error handling with AWS-specific error messages
- AWS credential chain integration
- Press `c` to copy selected resource's info to clipboard

**🚧 Planned:**

- Additional AWS services (Route53, EC2, ELB, S3, Lambda, CloudWatch)
  - _Each new service only needs to implement the `AwsService` trait_
- Help system (`:help`)
- Resource operations (start/stop, etc.)

**📋 Architecture Benefits:**

- **Plugin Architecture**: Adding new AWS services requires only implementing
  `AwsService` trait
- **Type Safety**: Generic framework with runtime type safety via downcasting
- **Clean Separation**: Services are self-contained with their own rendering
  logic
- **Extensible**: Framework supports custom view types and service-specific
  features

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
- `async-trait` - Async traits support for service framework

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
- **Extensibility**: Plugin-based architecture for easy service addition
- **Maintainability**: Clean separation between framework and service
  implementations

## Service Development Guide

### Adding a New AWS Service

To add a new AWS service (e.g., Route53), implement the `AwsService` trait:

```rust
pub struct Route53Service {
    client: aws_sdk_route53::Client,
}

#[async_trait]
impl AwsService for Route53Service {
    fn metadata(&self) -> ServiceMetadata {
        ServiceMetadata {
            id: "route53".to_string(),
            name: "Route 53".to_string(),
            description: "AWS DNS service".to_string(),
            command: "route53".to_string(),
        }
    }

    async fn load_data(&self, view_state: &ViewState) -> Result<ResourceData> {
        // Load Route53 hosted zones or records
    }

    fn render(&self, f: &mut Frame, area: Rect, app: &App, view_state: &ViewState, data: &ResourceData) {
        // Render Route53-specific UI
    }

    // ... implement other required methods
}
```

### Resource Types

Each service resource must implement `ResourceItem`:

```rust
#[derive(Clone, Debug)]
pub struct Route53HostedZone {
    pub zone_id: String,
    pub name: String,
    // ... other fields
}

impl ResourceItem for Route53HostedZone {
    fn id(&self) -> String { self.zone_id.clone() }
    fn as_any(&self) -> &dyn Any { self }
    fn clone_box(&self) -> Box<dyn ResourceItem> { Box::new(self.clone()) }
}
```

### Registration

Register the service in `main.rs`:

```rust
let route53_service = Route53Service::new(route53_client);
app.service_manager.register_service(Arc::new(route53_service));
```

The framework automatically handles:

- Command routing (`:route53`)
- Data loading and caching
- UI rendering dispatch
- Navigation and filtering
- Error handling and display
