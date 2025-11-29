{
  description = "Clipr - High-performance TUI clipboard history manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Rust toolchain with latest stable
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustToolchain

            # Clipboard integration (required)
            wl-clipboard

            # Build dependencies
            pkg-config

            # Development tools
            cargo-watch
            cargo-flamegraph
            hyperfine

            # For valgrind memory profiling
            valgrind

            # Optional: faster linker for quicker iteration
            mold
          ];

          # Environment variables
          RUST_BACKTRACE = "1";

          # Use mold linker for faster linking (iteration speed)
          RUSTFLAGS = "-C link-arg=-fuse-ld=mold";

          shellHook = ''
            echo "🎨 Clipr development environment"
            echo ""
            echo "Rust version: $(rustc --version)"
            echo "wl-clipboard: $(wl-paste --version 2>&1 || echo 'not available')"
            echo ""
            echo "Quick commands:"
            echo "  cargo run                  # Run clipr TUI"
            echo "  cargo run -- --daemon      # Run clipboard monitor daemon"
            echo "  cargo test                 # Run all tests"
            echo "  cargo bench                # Run benchmarks"
            echo "  cargo watch -x 'test --lib'  # Watch mode for tests"
            echo ""
            echo "Performance measurement:"
            echo "  hyperfine 'cargo run --release -- --help'  # Cold start benchmark"
            echo "  cargo flamegraph           # CPU profiling"
            echo ""
          '';
        };

        # Package definition (for building clipr)
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "clipr";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            wl-clipboard
          ];

          meta = with pkgs.lib; {
            description = "High-performance TUI clipboard history manager for Wayland";
            homepage = "https://github.com/yourusername/clipr";
            license = licenses.mit;
            maintainers = [ ];
          };
        };
      }
    );
}
