{
  description = "merle — RAW photo culler devShell";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          just
          cargo
          rustc
          rustfmt
          clippy
          cargo-nextest
          bacon
          rust-analyzer
          cargo-deny
          typos
        ];
      };
    };
}
