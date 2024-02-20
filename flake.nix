{
  description = "ndc-spec";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";

    nixpkgs.url = "github:NixOS/nixpkgs/master";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs =
    { self
    , flake-utils
    , nixpkgs
    , crane
    , rust-overlay
    }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };

      rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

      buildDependencies =
        [
          pkgs.openssl
          pkgs.pkg-config
        ];
      runtimeDependencies =
        pkgs.lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.CoreFoundation
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          pkgs.libiconv
        ];

      buildArgs = {
        pname = "ndc-spec";
        src =
          let
            isCsvFile = path: _type: builtins.match ".*\\.csv$" path != null;
            isJsonFile = path: _type: builtins.match ".*\\.json$" path != null;
            isJsonSchemaFile = path: _type: builtins.match ".*\\.jsonschema$" path != null;
            isSourceFile = path: type:
              isCsvFile path type
              || isJsonFile path type
              || isJsonSchemaFile path type
              || craneLib.filterCargoSources path type;
          in
          pkgs.lib.cleanSourceWith { src = craneLib.path ./.; filter = isSourceFile; };
        nativeBuildInputs = buildDependencies;
        buildInputs = runtimeDependencies;
      };
    in
    {
      packages.cargo-artifacts = craneLib.buildDepsOnly buildArgs;

      packages.default = craneLib.buildPackage (buildArgs // {
        cargoArtifacts = self.packages.${system}.cargo-artifacts;
      });

      apps.ndc-reference = flake-utils.lib.mkApp {
        drv = self.packages.${system}.default;
        exePath = "/bin/ndc-reference";
      };

      apps.ndc-test = flake-utils.lib.mkApp {
        drv = self.packages.${system}.default;
        exePath = "/bin/ndc-test";
      };

      devShells.default = pkgs.mkShell {
        nativeBuildInputs = [
          pkgs.cargo-edit
          pkgs.cargo-machete
          pkgs.cargo-nextest
          pkgs.cargo-watch
          rustToolchain

          pkgs.just
          pkgs.mdbook
        ] ++ buildDependencies;

        buildInputs = runtimeDependencies;
      };

      formatter = pkgs.nixpkgs-fmt;
    });
}
