# leptos-actix-realtime-test

A university project built to experiment with Rust web frameworks and their real-time rendering capabilities.

## Features

- Full-stack Rust web application
- Server-side rendering (SSR) with Leptos
- Client-side WebAssembly support
- Responsive design

## Tech Stack

- **Rust** - Systems programming language known for performance and safety
- **Leptos** - A reactive web framework for building web applications in Rust
- **Actix Web** - High-performance web framework for Rust
- **WebAssembly** - Running Rust on the browser
- **SCSS** - For styling

## Getting Started

### Prerequisites

- Rust toolchain (latest stable version)
- Cargo (comes with Rust)
- cargo-leptos (for development workflow)

### Installation

1. Clone the repository

```bash
git clone https://github.com/salvijusbieliauskas/leptos-actix-realtime-test.git
cd leptos-actix-realtime-test
```

2. Install cargo-leptos

```bash
cargo install cargo-leptos
```

3. Run the development server

```bash
cargo leptos watch
```

The server will start at http://0.0.0.0:3000

### Building for Production

```bash
cargo leptos build --release
```

## Project Structure

- `src/` - Rust source code
  - `main.rs` - Server entrypoint with Actix Web setup
  - `app.rs` - Main Leptos application component
  - `lib.rs` - Library configuration and exports
- `style/` - SCSS stylesheets
- `assets/` - Static assets

## Configuration

The project uses feature flags to enable different compilation modes:
- `ssr` - Server-side rendering
- `hydrate` - Client-side hydration
- `csr` - Client-side rendering

See the `Cargo.toml` file for detailed configuration options.

## Considerations

Real-time web applications should use websockets as opposed to polling, but a university side-project did not merit the
effort required to implement a websocket connection meaningfully.