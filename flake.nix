{
  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.x86_64-linux;
    set = import ./default.nix { inherit pkgs; };
  in {
    packages.${system} = set;
    app = set;
  };
}
