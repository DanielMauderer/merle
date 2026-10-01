{
  description = "merle — RAW photo culler devShell";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};

      # winit/softbuffer dlopen these at runtime, so they must be on the
      # library path rather than just available at link time.
      runtimeLibs = with pkgs; [
        wayland
        libxkbcommon
        libGL
        libX11
        libxcursor
        libxi
        libxrandr
      ];
    in
    {
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
