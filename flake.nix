{
  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.x86_64-linux;
  in {
    packages.${system} = import ./default.nix { inherit pkgs; };
    app = import ./default.nix { inherit pkgs; };
  };
}
