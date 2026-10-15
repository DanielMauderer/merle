{
  description = "merle — RAW photo culler";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

      # winit/wgpu dlopen these at runtime, so they must be on the
      # library path rather than just available at link time.
      runtimeLibs = with pkgs; [
        wayland
        libxkbcommon
        libGL
        vulkan-loader
        libX11
        libxcursor
        libxi
        libxrandr
      ];
    in
    {
      packages.${system} = {
        merle-view = pkgs.rustPlatform.buildRustPackage {
          pname = "merle-view";
          inherit (cargoToml.workspace.package) version;
          src = pkgs.lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;

          cargoBuildFlags = [ "-p" "merle-view" ];
          cargoTestFlags = [ "-p" "merle-view" ];

          # dlopen'd libraries are invisible to the linker, so add them to
          # the binary's RUNPATH directly.
          postFixup = ''
            patchelf --add-rpath ${pkgs.lib.makeLibraryPath runtimeLibs} $out/bin/merle-view
          '';

          meta = {
            description = "Fast-start RAW photo viewer";
            homepage = cargoToml.workspace.package.repository;
            license = pkgs.lib.licenses.gpl3Plus;
            mainProgram = "merle-view";
            platforms = [ system ];
          };
        };
        default = self.packages.${system}.merle-view;
      };

      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          just
          curl
          cargo
          rustc
          rustfmt
          clippy
          cargo-nextest
          bacon
          rust-analyzer
          cargo-deny
          typos
          pkg-config
        ];

        buildInputs = runtimeLibs;

        shellHook = ''
          export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath runtimeLibs}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
        '';
      };
    };
}
