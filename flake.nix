{
  description = "A dev flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    pkgs = nixpkgs.legacyPackages."x86_64-linux";
  in {
    devShells."x86_64-linux".default = pkgs.mkShell {
      LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${with pkgs;
        lib.makeLibraryPath [
          fontconfig
          freetype
          wayland
        ]}";

      buildInputs = with pkgs; [
        fontconfig
        freetype
        pkg-config
        rustup
      ];
    };
  };
}
